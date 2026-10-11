//! People and the signed-in person's presence: directory search, new
//! chats, availability and status message.

use serde_json::json;

use super::types as g;
use super::{Graph, GraphError, convert, segment};
use crate::model::{Availability, Chat, Member, Presence};

impl Graph {
    /// Directory search by name or email (User.ReadBasic.All). `$search`
    /// on users needs the `ConsistencyLevel: eventual` header.
    pub async fn search_people(&self, query: &str) -> Result<Vec<Member>, GraphError> {
        // Quotes and backslashes would end the search clause early.
        let cleaned: String = query
            .chars()
            .filter(|c| !matches!(c, '"' | '\\'))
            .collect::<String>()
            .trim()
            .to_owned();
        if cleaned.chars().count() < 2 {
            return Ok(Vec::new());
        }
        let search = format!("\"displayName:{cleaned}\" OR \"mail:{cleaned}\"");
        let url = format!(
            "users?$search={}&$select=id,displayName,mail,userPrincipalName&$top=15&$count=true",
            urlencoding::encode(&search)
        );
        let page: g::Page<g::User> = self
            .get_with(&url, &[("ConsistencyLevel", "eventual")])
            .await?;
        Ok(page
            .value
            .into_iter()
            .map(|user| Member {
                email: user.mail.or(user.user_principal_name),
                display_name: user.display_name.unwrap_or_default(),
                user_id: user.id,
            })
            .filter(|member| !member.display_name.is_empty())
            .collect())
    }

    /// `POST /chats`: a one-on-one chat with one other person, or a group
    /// chat with several (Chat.Create). Graph returns the existing chat
    /// when a one-on-one chat with that person already exists.
    pub async fn create_chat(
        &self,
        me: &str,
        others: &[String],
        topic: Option<&str>,
    ) -> Result<Chat, GraphError> {
        let member = |id: &str| {
            json!({
                "@odata.type": "#microsoft.graph.aadUserConversationMember",
                "roles": ["owner"],
                "user@odata.bind": format!("https://graph.microsoft.com/v1.0/users('{id}')"),
            })
        };
        let mut members = vec![member(me)];
        members.extend(others.iter().map(|id| member(id)));
        let mut body = json!({
            "chatType": if others.len() == 1 { "oneOnOne" } else { "group" },
            "members": members,
        });
        if others.len() > 1
            && let Some(topic) = topic.map(str::trim).filter(|t| !t.is_empty())
        {
            body["topic"] = json!(topic);
        }
        let created: g::Chat = self.post("chats", &body).await?;
        // The answer has no members; read the chat back with them.
        match self
            .get::<g::Chat>(&format!("chats/{}?$expand=members", segment(&created.id)))
            .await
        {
            Ok(full) => Ok(convert::chat(full)),
            Err(_) => Ok(convert::chat(created)),
        }
    }

    /// `GET /me/presence` (Presence.Read, part of Presence.Read.All).
    pub async fn my_presence(&self) -> Result<Presence, GraphError> {
        let raw: g::Presence = self.get("me/presence").await?;
        Ok(convert::presence(raw).1)
    }

    /// `setUserPreferredPresence`, or `clearUserPreferredPresence` for
    /// `None` (Presence.ReadWrite). It shows while the person has any
    /// presence session: a Teams client, or Wren's own.
    pub async fn set_preferred_presence(
        &self,
        me: &str,
        availability: Option<Availability>,
    ) -> Result<(), GraphError> {
        match availability.and_then(Availability::graph_preferred) {
            Some((availability, activity)) => {
                self.post_no_content(
                    &format!("users/{}/presence/setUserPreferredPresence", segment(me)),
                    &json!({ "availability": availability, "activity": activity }),
                )
                .await
            }
            None => {
                self.post_no_content(
                    &format!("users/{}/presence/clearUserPreferredPresence", segment(me)),
                    &json!({}),
                )
                .await
            }
        }
    }

    /// `setStatusMessage`; an empty text clears it (Presence.ReadWrite).
    pub async fn set_status_message(&self, me: &str, text: &str) -> Result<(), GraphError> {
        self.post_no_content(
            &format!("users/{}/presence/setStatusMessage", segment(me)),
            &json!({ "statusMessage": { "message": { "content": text.trim(), "contentType": "text" } } }),
        )
        .await
    }

    /// `setPresence`: Wren's own presence session, as a Teams client keeps
    /// one while it runs. `sessionId` is the app's client ID.
    pub async fn set_presence_session(
        &self,
        me: &str,
        client_id: &str,
        away: bool,
        minutes: u32,
    ) -> Result<(), GraphError> {
        let (availability, activity) = if away {
            ("Away", "Away")
        } else {
            ("Available", "Available")
        };
        self.post_no_content(
            &format!("users/{}/presence/setPresence", segment(me)),
            &json!({
                "sessionId": client_id,
                "availability": availability,
                "activity": activity,
                "expirationDuration": format!("PT{}M", minutes.clamp(5, 240)),
            }),
        )
        .await
    }

    /// `clearPresence` for Wren's session.
    pub async fn clear_presence_session(
        &self,
        me: &str,
        client_id: &str,
    ) -> Result<(), GraphError> {
        self.post_no_content(
            &format!("users/{}/presence/clearPresence", segment(me)),
            &json!({ "sessionId": client_id }),
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

    async fn graph(server: &MockServer) -> Graph {
        Graph::with_base(
            reqwest::Client::new(),
            testing::session("at", &server.uri()).await,
            &server.uri(),
        )
    }

    #[tokio::test]
    async fn people_search_is_eventual_and_escaped() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/users"))
            .and(header("consistencylevel", "eventual"))
            .and(query_param("$search", "\"displayName:ada\" OR \"mail:ada\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"value": [
                {"id": "u-ada", "displayName": "Ada Lovelace", "mail": "ada@contoso.com"},
                {"id": "u-x", "displayName": "Ada Room", "userPrincipalName": "room@contoso.com"}
            ]})))
            .expect(1)
            .mount(&server)
            .await;
        let graph = graph(&server).await;
        let people = graph.search_people("a\"da").await.unwrap();
        assert_eq!(people.len(), 2);
        assert_eq!(people[1].email.as_deref(), Some("room@contoso.com"));
        assert!(graph.search_people("a").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn new_chats_include_me_and_bind_users() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chats"))
            .and(body_json(serde_json::json!({
                "chatType": "group",
                "topic": "Launch",
                "members": [
                    {"@odata.type": "#microsoft.graph.aadUserConversationMember", "roles": ["owner"], "user@odata.bind": "https://graph.microsoft.com/v1.0/users('u-me')"},
                    {"@odata.type": "#microsoft.graph.aadUserConversationMember", "roles": ["owner"], "user@odata.bind": "https://graph.microsoft.com/v1.0/users('u-ada')"},
                    {"@odata.type": "#microsoft.graph.aadUserConversationMember", "roles": ["owner"], "user@odata.bind": "https://graph.microsoft.com/v1.0/users('u-grace')"}
                ]
            })))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"id": "19:new@thread.v2", "chatType": "group", "topic": "Launch"})))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/chats/19:new@thread.v2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "19:new@thread.v2", "chatType": "group", "topic": "Launch",
                "members": [{"userId": "u-me", "displayName": "Me"}, {"userId": "u-ada", "displayName": "Ada"}, {"userId": "u-grace", "displayName": "Grace"}]
            })))
            .mount(&server)
            .await;
        let chat = graph(&server)
            .await
            .create_chat("u-me", &["u-ada".into(), "u-grace".into()], Some("Launch"))
            .await
            .unwrap();
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.title("u-me"), "Launch");
    }

    #[tokio::test]
    async fn presence_writes_hit_their_endpoints() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/users/u-me/presence/setUserPreferredPresence"))
            .and(body_json(
                serde_json::json!({"availability": "DoNotDisturb", "activity": "DoNotDisturb"}),
            ))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/users/u-me/presence/clearUserPreferredPresence"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/users/u-me/presence/setStatusMessage"))
            .and(body_json(serde_json::json!({"statusMessage": {"message": {"content": "Back at 3", "contentType": "text"}}})))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/users/u-me/presence/setPresence"))
            .and(body_json(serde_json::json!({"sessionId": "client", "availability": "Available", "activity": "Available", "expirationDuration": "PT30M"})))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        let graph = graph(&server).await;
        graph
            .set_preferred_presence("u-me", Some(Availability::DoNotDisturb))
            .await
            .unwrap();
        graph.set_preferred_presence("u-me", None).await.unwrap();
        graph
            .set_status_message("u-me", " Back at 3 ")
            .await
            .unwrap();
        graph
            .set_presence_session("u-me", "client", false, 30)
            .await
            .unwrap();
    }
}
