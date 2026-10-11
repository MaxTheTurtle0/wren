//! Sending files: upload to the sender's OneDrive, share with the
//! conversation, and reference the file in the message (Files.ReadWrite).
//!
//! Files go to "Microsoft Teams Chat Files", the folder Teams itself uses
//! for chat attachments. Up to 4 MiB uploads in one request; larger files
//! go through an upload session in 5 MiB chunks (a multiple of 320 KiB, as
//! Graph requires). Chunks are sent to the session's pre-authenticated URL
//! without the access token.

use std::path::Path;

use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use super::types::{DriveItem, UploadSession};
use super::{Graph, GraphError, segment};

/// The folder Teams keeps chat attachments in.
pub const FOLDER: &str = "Microsoft Teams Chat Files";
/// Files up to this size go up in one request.
pub const SIMPLE_LIMIT: u64 = 4 * 1024 * 1024;
/// 16 × 320 KiB.
pub const CHUNK: u64 = 16 * 320 * 1024;
/// The largest file Wren sends.
pub const MAX_FILE: u64 = 1024 * 1024 * 1024;

/// A file attached to an outgoing message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedFile {
    pub id: String,
    pub name: String,
    pub url: String,
}

/// A file name OneDrive accepts: no path separators or reserved characters.
fn safe_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if matches!(
                c,
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '#' | '%'
            ) {
                '_'
            } else {
                c
            }
        })
        .collect::<String>()
        .trim()
        .to_owned()
}

/// The path segment for a file name: percent-encoded, no slashes.
fn file_segment(name: &str) -> String {
    urlencoding::encode(&safe_name(name)).into_owned()
}

fn item_path(name: &str) -> String {
    format!(
        "me/drive/root:/{}/{}",
        urlencoding::encode(FOLDER),
        file_segment(name)
    )
}

impl Graph {
    /// Uploads a file to the person's OneDrive. Existing names are kept;
    /// the new file is renamed instead (`conflictBehavior=rename`).
    pub async fn upload_file(&self, path: &Path) -> Result<DriveItem, GraphError> {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| "file".into());
        let size = tokio::fs::metadata(path)
            .await
            .map_err(|_| GraphError::Network("the file could not be read".into()))?
            .len();
        if size > MAX_FILE {
            return Err(GraphError::Http {
                status: 413,
                code: "fileTooLarge".into(),
            });
        }
        let uploaded = if size <= SIMPLE_LIMIT {
            let bytes = tokio::fs::read(path)
                .await
                .map_err(|_| GraphError::Network("the file could not be read".into()))?;
            self.put_bytes::<DriveItem>(
                &format!(
                    "{}:/content?@microsoft.graph.conflictBehavior=rename",
                    item_path(&name)
                ),
                &bytes,
            )
            .await?
        } else {
            let session: UploadSession = self
                .post(
                    &format!("{}:/createUploadSession", item_path(&name)),
                    &json!({ "item": { "@microsoft.graph.conflictBehavior": "rename", "name": safe_name(&name) } }),
                )
                .await?;
            self.upload_chunks(&session.upload_url, path, size).await?
        };
        // The attachment must point at webDavUrl, which uploads do not
        // return unless asked for.
        self.get::<DriveItem>(&format!(
            "me/drive/items/{}?$select=id,name,eTag,webUrl,webDavUrl,size",
            segment(&uploaded.id)
        ))
        .await
        .or(Ok(uploaded))
    }

    async fn upload_chunks(
        &self,
        upload_url: &str,
        path: &Path,
        size: u64,
    ) -> Result<DriveItem, GraphError> {
        let unreadable = || GraphError::Network("the file could not be read".into());
        let mut file = tokio::fs::File::open(path)
            .await
            .map_err(|_| unreadable())?;
        let mut start = 0u64;
        let mut buffer = vec![0u8; CHUNK as usize];
        loop {
            let length = CHUNK.min(size - start);
            file.seek(std::io::SeekFrom::Start(start))
                .await
                .map_err(|_| unreadable())?;
            file.read_exact(&mut buffer[..length as usize])
                .await
                .map_err(|_| unreadable())?;
            let end = start + length - 1;
            // The upload URL is pre-authenticated: no Authorization header.
            let response = self
                .plain_http()
                .put(upload_url)
                .header(
                    reqwest::header::CONTENT_RANGE,
                    format!("bytes {start}-{end}/{size}"),
                )
                .header(reqwest::header::CONTENT_LENGTH, length)
                .body(buffer[..length as usize].to_vec())
                .timeout(std::time::Duration::from_secs(120))
                .send()
                .await
                .map_err(|error| {
                    GraphError::Network(crate::auth::token::transport_reason(&error))
                })?;
            let status = response.status();
            if !status.is_success() {
                return Err(GraphError::Http {
                    status: status.as_u16(),
                    code: "uploadFailed".into(),
                });
            }
            start = end + 1;
            if start >= size {
                // The last chunk's answer is the finished item.
                return response
                    .json::<DriveItem>()
                    .await
                    .map_err(|_| GraphError::Decode);
            }
        }
    }

    /// Shares an uploaded file with specific people (the chat's members),
    /// read-only, without an invitation email.
    pub async fn share_with(&self, item: &str, emails: &[String]) -> Result<(), GraphError> {
        if emails.is_empty() {
            return Ok(());
        }
        let recipients: Vec<serde_json::Value> = emails
            .iter()
            .map(|email| json!({ "email": email }))
            .collect();
        self.post_no_content(
            &format!("me/drive/items/{}/invite", segment(item)),
            &json!({
                "recipients": recipients,
                "roles": ["read"],
                "requireSignIn": true,
                "sendInvitation": false,
            }),
        )
        .await
    }

    /// A view link for everyone in the organisation, for files posted in
    /// channels (whose members Wren does not list).
    pub async fn organization_link(&self, item: &str) -> Result<(), GraphError> {
        self.post_no_content(
            &format!("me/drive/items/{}/createLink", segment(item)),
            &json!({ "type": "view", "scope": "organization" }),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing;
    use super::*;
    use wiremock::matchers::{body_json, header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn attachment_ids_come_from_the_etag() {
        let item = DriveItem {
            e_tag: Some("\"{153FA47D-18C9-4179-BE08-9879815A9F90},1\"".into()),
            ..Default::default()
        };
        assert_eq!(
            item.attachment_id().as_deref(),
            Some("153fa47d-18c9-4179-be08-9879815a9f90")
        );
        assert_eq!(DriveItem::default().attachment_id(), None);
    }

    #[test]
    fn file_names_cannot_escape_the_folder() {
        assert_eq!(
            item_path("a/b?.txt"),
            "me/drive/root:/Microsoft%20Teams%20Chat%20Files/a_b_.txt"
        );
        assert_eq!(
            item_path("Q3 plan.pdf"),
            "me/drive/root:/Microsoft%20Teams%20Chat%20Files/Q3%20plan.pdf"
        );
    }

    async fn graph(server: &MockServer) -> Graph {
        Graph::with_base(
            reqwest::Client::new(),
            testing::session("at", &server.uri()).await,
            &server.uri(),
        )
    }

    #[tokio::test]
    async fn small_files_go_up_in_one_request() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/me/drive/root:/Microsoft%20Teams%20Chat%20Files/notes.txt:/content"))
            .and(query_param("@microsoft.graph.conflictBehavior", "rename"))
            .and(header("authorization", "Bearer at"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "id": "item-1", "name": "notes.txt", "eTag": "\"{153FA47D-18C9-4179-BE08-9879815A9F90},1\"",
                "webUrl": "https://contoso-my.sharepoint.com/personal/me/Documents/notes.txt"
            })))
            .expect(1)
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("notes.txt");
        std::fs::write(&file, b"hello").unwrap();
        let item = graph(&server).await.upload_file(&file).await.unwrap();
        assert_eq!(item.id, "item-1");
        assert!(item.attachment_id().is_some());
    }

    #[tokio::test]
    async fn large_files_use_an_upload_session_without_the_token() {
        let server = MockServer::start().await;
        let upload_url = format!("{}/upload-session/abc", server.uri());
        Mock::given(method("POST"))
            .and(path(
                "/me/drive/root:/Microsoft%20Teams%20Chat%20Files/big.bin:/createUploadSession",
            ))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({ "uploadUrl": upload_url })),
            )
            .expect(1)
            .mount(&server)
            .await;
        let size = CHUNK + 1000;
        Mock::given(method("PUT"))
            .and(path("/upload-session/abc"))
            .and(header(
                "content-range",
                format!("bytes 0-{}/{size}", CHUNK - 1).as_str(),
            ))
            .respond_with(
                ResponseTemplate::new(202).set_body_json(
                    serde_json::json!({ "nextExpectedRanges": [format!("{CHUNK}-")] }),
                ),
            )
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("PUT"))
            .and(path("/upload-session/abc"))
            .and(header(
                "content-range",
                format!("bytes {CHUNK}-{}/{size}", size - 1).as_str(),
            ))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({ "id": "item-2", "name": "big.bin" })),
            )
            .expect(1)
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("big.bin");
        std::fs::write(&file, vec![7u8; size as usize]).unwrap();
        let item = graph(&server).await.upload_file(&file).await.unwrap();
        assert_eq!(item.id, "item-2");
        for request in server.received_requests().await.unwrap() {
            if request.url.path() == "/upload-session/abc" {
                assert!(
                    !request.headers.contains_key("authorization"),
                    "the token went to the upload URL"
                );
            }
        }
    }

    #[tokio::test]
    async fn sharing_invites_members_read_only() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/me/drive/items/item-1/invite"))
            .and(body_json(serde_json::json!({
                "recipients": [{"email": "ada@contoso.com"}],
                "roles": ["read"], "requireSignIn": true, "sendInvitation": false
            })))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"value": []})),
            )
            .expect(1)
            .mount(&server)
            .await;
        graph(&server)
            .await
            .share_with("item-1", &["ada@contoso.com".into()])
            .await
            .unwrap();
    }
}
