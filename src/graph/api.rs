//! The Graph endpoints Wren uses, each with the delegated permission it
//! needs. docs/permissions.md lists them for administrators.

use serde_json::json;

use super::types as g;
use super::{Graph, GraphError, convert, segment};
use crate::model::{
    Channel, Chat, ConversationId, Meeting, Message, Presence, Profile, SearchHit, Team,
};

/// Graph's page size limit for chats and messages.
pub const PAGE: usize = 50;
/// Avatars and inline images larger than this are not downloaded.
pub const MEDIA_LIMIT: usize = 8 * 1024 * 1024;

/// A message to send: its HTML, the people it mentions (`<at id="n">` in
/// the HTML), and files already uploaded and shared.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outgoing {
    pub html: String,
    pub mentions: Vec<crate::model::MentionRef>,
    pub files: Vec<super::files::SharedFile>,
}

impl Outgoing {
    pub fn html(html: impl Into<String>) -> Self {
        Self {
            html: html.into(),
            ..Default::default()
        }
    }

    /// The `chatMessage` body Graph takes.
    pub fn body(&self) -> serde_json::Value {
        let mut content = self.html.clone();
        for file in &self.files {
            content.push_str(&format!("<attachment id=\"{}\"></attachment>", file.id));
        }
        let mut body = json!({ "body": { "contentType": "html", "content": content } });
        if !self.mentions.is_empty() {
            body["mentions"] = self
                .mentions
                .iter()
                .enumerate()
                .map(|(id, mention)| {
                    json!({
                        "id": id,
                        "mentionText": mention.name,
                        "mentioned": { "user": {
                            "id": mention.user_id,
                            "displayName": mention.name,
                            "userIdentityType": "aadUser",
                        }},
                    })
                })
                .collect();
        }
        if !self.files.is_empty() {
            body["attachments"] = self
                .files
                .iter()
                .map(|file| {
                    json!({
                        "id": file.id,
                        "contentType": "reference",
                        "contentUrl": file.url,
                        "name": file.name,
                    })
                })
                .collect();
        }
        body
    }
}

/// A page of messages, newest first, and the link to the older ones.
#[derive(Debug, Default)]
pub struct MessagePage {
    pub messages: Vec<Message>,
    pub older: Option<String>,
}

impl Graph {
    /// `GET /me` (User.Read).
    pub async fn me(&self, tenant_id: Option<String>) -> Result<Profile, GraphError> {
        let user: g::User = self
            .get("me?$select=id,displayName,mail,userPrincipalName,jobTitle")
            .await?;
        Ok(convert::profile(user, tenant_id))
    }

    /// `GET /me/chats` with members and the newest message, newest activity
    /// first (Chat.ReadWrite). Follows up to `pages` pages of 50.
    ///
    /// Graph documents both expansions but no example combines them; if it
    /// refuses the combination, the members and the previews are fetched in
    /// two listings and joined by chat id.
    pub async fn chats(&self, pages: usize) -> Result<Vec<Chat>, GraphError> {
        let combined = format!(
            "me/chats?$expand=members,lastMessagePreview&$orderby=lastMessagePreview/createdDateTime%20desc&$top={PAGE}"
        );
        match self.chat_pages(&combined, pages).await {
            Err(GraphError::Http { status: 400, .. }) => {}
            other => return other.map(|chats| chats.into_iter().map(convert::chat).collect()),
        }
        log::info!(
            "Graph refused the combined chat listing; listing members and previews separately"
        );
        let mut previews = self
            .chat_pages(
                &format!("me/chats?$expand=lastMessagePreview&$orderby=lastMessagePreview/createdDateTime%20desc&$top={PAGE}"),
                pages,
            )
            .await?;
        let members = self
            .chat_pages(&format!("me/chats?$expand=members&$top={PAGE}"), pages + 2)
            .await?;
        let mut members: std::collections::HashMap<String, Vec<g::ConversationMember>> = members
            .into_iter()
            .filter_map(|chat| Some((chat.id, chat.members?)))
            .collect();
        for chat in &mut previews {
            chat.members = members.remove(&chat.id);
        }
        Ok(previews.into_iter().map(convert::chat).collect())
    }

    async fn chat_pages(&self, first: &str, pages: usize) -> Result<Vec<g::Chat>, GraphError> {
        let mut url = first.to_owned();
        let mut chats = Vec::new();
        for _ in 0..pages.max(1) {
            let page: g::Page<g::Chat> = self.get(&url).await?;
            chats.extend(page.value);
            match page.next_link {
                Some(next) => url = next,
                None => break,
            }
        }
        Ok(chats)
    }

    /// One chat, for a chat Wren has not listed yet (Chat.ReadWrite).
    pub async fn chat(&self, chat: &str) -> Result<Chat, GraphError> {
        let raw: g::Chat = self
            .get(&format!(
                "chats/{}?$expand=members,lastMessagePreview",
                segment(chat)
            ))
            .await?;
        Ok(convert::chat(raw))
    }

    /// A chat's newest messages, or the 50 created before `before`
    /// (Chat.ReadWrite). The cursor is a timestamp, so paging back works
    /// from the cache too.
    pub async fn chat_messages(
        &self,
        chat: &str,
        before: Option<i64>,
    ) -> Result<MessagePage, GraphError> {
        let conversation = ConversationId::Chat(chat.to_owned());
        let mut url = format!(
            "chats/{}/messages?$top={PAGE}&$orderby=createdDateTime%20desc",
            segment(chat)
        );
        if let Some(before) = before {
            url.push_str(&format!(
                "&$filter=createdDateTime%20lt%20{}",
                crate::time::to_iso(before)
            ));
        }
        let page: g::Page<g::ChatMessage> = self.get(&url).await?;
        Ok(MessagePage {
            messages: page
                .value
                .into_iter()
                .map(|m| convert::message(m, Some(&conversation)))
                .collect(),
            older: page.next_link,
        })
    }

    /// Messages created, edited, reacted to or deleted after `since`: the
    /// date-ranged refresh the Teams API's polling rules ask for
    /// (Chat.ReadWrite).
    pub async fn chat_changes(&self, chat: &str, since: i64) -> Result<Vec<Message>, GraphError> {
        let conversation = ConversationId::Chat(chat.to_owned());
        let since = crate::time::to_iso(since);
        let mut url = format!(
            "chats/{}/messages?$top={PAGE}&$orderby=lastModifiedDateTime%20desc&$filter=lastModifiedDateTime%20gt%20{since}",
            segment(chat)
        );
        let mut messages = Vec::new();
        // A burst larger than a few pages is reloaded as a fresh page instead.
        for _ in 0..4 {
            let page: g::Page<g::ChatMessage> = self.get(&url).await?;
            messages.extend(
                page.value
                    .into_iter()
                    .map(|m| convert::message(m, Some(&conversation))),
            );
            match page.next_link {
                Some(next) => url = next,
                None => break,
            }
        }
        Ok(messages)
    }

    /// `POST /chats/{id}/messages` (ChatMessage.Send).
    pub async fn send_chat_message(
        &self,
        chat: &str,
        message: &Outgoing,
    ) -> Result<Message, GraphError> {
        let raw: g::ChatMessage = self
            .post(
                &format!("chats/{}/messages", segment(chat)),
                &message.body(),
            )
            .await?;
        Ok(convert::message(
            raw,
            Some(&ConversationId::Chat(chat.to_owned())),
        ))
    }

    /// `POST /chats/{id}/messages/replyWithQuote` (ChatMessage.Send).
    pub async fn reply_with_quote(
        &self,
        chat: &str,
        quoted: &str,
        message: &Outgoing,
    ) -> Result<Message, GraphError> {
        let raw: g::ChatMessage = self
            .post(
                &format!("chats/{}/messages/replyWithQuote", segment(chat)),
                &json!({ "messageIds": [quoted], "replyMessage": message.body() }),
            )
            .await?;
        Ok(convert::message(
            raw,
            Some(&ConversationId::Chat(chat.to_owned())),
        ))
    }

    /// `PATCH` a message's body: chats need Chat.ReadWrite, channels
    /// ChannelMessage.ReadWrite (admin consent).
    pub async fn edit_message(
        &self,
        conversation: &ConversationId,
        message: &str,
        html: &str,
    ) -> Result<(), GraphError> {
        self.patch(
            &message_path(conversation, message),
            &json!({ "body": { "contentType": "html", "content": html } }),
        )
        .await
    }

    /// `softDelete` (Chat.ReadWrite; channels need ChannelMessage.ReadWrite).
    pub async fn delete_message(
        &self,
        me: &str,
        conversation: &ConversationId,
        message: &str,
    ) -> Result<(), GraphError> {
        let path = match conversation {
            ConversationId::Chat(chat) => format!(
                "users/{}/chats/{}/messages/{}/softDelete",
                segment(me),
                segment(chat),
                segment(message)
            ),
            ConversationId::Channel { .. } => {
                format!("{}/softDelete", message_path(conversation, message))
            }
        };
        self.post_no_content(&path, &json!({})).await
    }

    /// `setReaction` / `unsetReaction` with a Unicode emoji
    /// (ChatMessage.Send, ChannelMessage.Send).
    pub async fn react(
        &self,
        conversation: &ConversationId,
        message: &str,
        emoji: &str,
        on: bool,
    ) -> Result<(), GraphError> {
        let action = if on { "setReaction" } else { "unsetReaction" };
        self.post_no_content(
            &format!("{}/{action}", message_path(conversation, message)),
            &json!({ "reactionType": emoji }),
        )
        .await
    }

    /// `markChatReadForUser` (Chat.ReadWrite).
    pub async fn mark_chat_read(
        &self,
        chat: &str,
        me: &str,
        tenant: &str,
    ) -> Result<(), GraphError> {
        self.post_no_content(
            &format!("chats/{}/markChatReadForUser", segment(chat)),
            &json!({ "user": { "id": me, "tenantId": tenant } }),
        )
        .await
    }

    /// `GET /me/joinedTeams` (Team.ReadBasic.All). The endpoint takes no
    /// OData parameters, so nothing is selected or filtered here.
    pub async fn joined_teams(&self) -> Result<Vec<Team>, GraphError> {
        let page: g::Page<g::Team> = self.get("me/joinedTeams").await?;
        let mut teams: Vec<Team> = page.value.into_iter().map(convert::team).collect();
        teams.sort_by_key(|team| team.name.to_lowercase());
        Ok(teams)
    }

    /// `GET /teams/{id}/allChannels`, shared channels included
    /// (Channel.ReadBasic.All).
    pub async fn channels(&self, team: &str) -> Result<Vec<Channel>, GraphError> {
        let page: g::Page<g::Channel> = self
            .get(&format!(
                "teams/{}/allChannels?$select=id,displayName,description,membershipType,webUrl",
                segment(team)
            ))
            .await?;
        let mut channels: Vec<Channel> = page
            .value
            .into_iter()
            .map(|c| convert::channel(c, team))
            .collect();
        // General first, as Teams lists it, then by name.
        channels.sort_by_key(|channel| (channel.name != "General", channel.name.to_lowercase()));
        Ok(channels)
    }

    /// A channel's newest posts with their replies (ChannelMessage.Read.All,
    /// which needs admin consent).
    pub async fn channel_messages(
        &self,
        team: &str,
        channel: &str,
        older: Option<&str>,
    ) -> Result<MessagePage, GraphError> {
        let conversation = ConversationId::Channel {
            team: team.to_owned(),
            channel: channel.to_owned(),
        };
        let url = match older {
            Some(link) => link.to_owned(),
            None => format!(
                "teams/{}/channels/{}/messages?$top=25&$expand=replies",
                segment(team),
                segment(channel)
            ),
        };
        let page: g::Page<g::ChatMessage> = self.get(&url).await?;
        Ok(MessagePage {
            messages: page
                .value
                .into_iter()
                .map(|m| convert::message(m, Some(&conversation)))
                .collect(),
            older: page.next_link,
        })
    }

    /// A post's replies (ChannelMessage.Read.All).
    pub async fn channel_replies(
        &self,
        team: &str,
        channel: &str,
        root: &str,
    ) -> Result<Vec<Message>, GraphError> {
        let conversation = ConversationId::Channel {
            team: team.to_owned(),
            channel: channel.to_owned(),
        };
        let page: g::Page<g::ChatMessage> = self
            .get(&format!(
                "teams/{}/channels/{}/messages/{}/replies?$top={PAGE}",
                segment(team),
                segment(channel),
                segment(root)
            ))
            .await?;
        let mut replies: Vec<Message> = page
            .value
            .into_iter()
            .map(|m| convert::message(m, Some(&conversation)))
            .collect();
        replies.sort_by_key(|m| m.created);
        Ok(replies)
    }

    /// A new post, or a reply to `root` (ChannelMessage.Send).
    pub async fn send_channel_message(
        &self,
        team: &str,
        channel: &str,
        root: Option<&str>,
        message: &Outgoing,
    ) -> Result<Message, GraphError> {
        let conversation = ConversationId::Channel {
            team: team.to_owned(),
            channel: channel.to_owned(),
        };
        let mut path = format!(
            "teams/{}/channels/{}/messages",
            segment(team),
            segment(channel)
        );
        if let Some(root) = root {
            path.push_str(&format!("/{}/replies", segment(root)));
        }
        let raw: g::ChatMessage = self.post(&path, &message.body()).await?;
        Ok(convert::message(raw, Some(&conversation)))
    }

    /// `POST /communications/getPresencesByUserId`, up to 650 people per
    /// call (Presence.Read.All).
    pub async fn presences(&self, users: &[String]) -> Result<Vec<(String, Presence)>, GraphError> {
        let mut all = Vec::new();
        for chunk in users.chunks(650) {
            let page: g::Page<g::Presence> = self
                .post(
                    "communications/getPresencesByUserId",
                    &json!({ "ids": chunk }),
                )
                .await?;
            all.extend(page.value.into_iter().map(convert::presence));
        }
        Ok(all)
    }

    /// The calendar between two instants, in UTC (Calendars.Read).
    pub async fn calendar(&self, from: i64, to: i64) -> Result<Vec<Meeting>, GraphError> {
        let url = format!(
            "me/calendarView?startDateTime={}&endDateTime={}&$top=100&$orderby=start/dateTime&$select=id,subject,bodyPreview,start,end,isAllDay,isCancelled,location,organizer,isOnlineMeeting,onlineMeeting,webLink",
            crate::time::to_iso(from),
            crate::time::to_iso(to)
        );
        let page: g::Page<g::Event> = self
            .get_with(&url, &[("Prefer", "outlook.timezone=\"UTC\"")])
            .await?;
        Ok(page
            .value
            .into_iter()
            .filter_map(convert::meeting)
            .collect())
    }

    /// A person's small photo, or `None` without one (User.ReadBasic.All).
    pub async fn photo(&self, user: &str) -> Result<Option<Vec<u8>>, GraphError> {
        match self
            .get_bytes(
                &format!("users/{}/photos/96x96/$value", segment(user)),
                super::api::MEDIA_LIMIT,
            )
            .await
        {
            Ok(bytes) => Ok(Some(bytes)),
            Err(GraphError::NotFound) => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Microsoft Search over chat messages (Chat.ReadWrite; channel posts
    /// also need ChannelMessage.Read.All).
    pub async fn search_messages(&self, query: &str) -> Result<Vec<SearchHit>, GraphError> {
        let response: g::SearchResponse = self
            .post(
                "search/query",
                &json!({ "requests": [{
                    "entityTypes": ["chatMessage"],
                    "query": { "queryString": query },
                    "from": 0,
                    "size": 25
                }]}),
            )
            .await?;
        let mut hits = Vec::new();
        for set in response.value {
            for container in set.hits_containers {
                for hit in container.hits {
                    if let Some(hit) = search_hit(hit) {
                        hits.push(hit);
                    }
                }
            }
        }
        Ok(hits)
    }
}

/// A Graph change-notification subscription.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: String,
    #[serde(default)]
    pub expiration_date_time: Option<String>,
}

impl Graph {
    /// `POST /subscriptions` for all of the person's chat messages, without
    /// resource data (Chat.Read; Chat.ReadWrite includes it).
    pub async fn subscribe_chat_messages(
        &self,
        me: &str,
        notification_url: &str,
        lifecycle_url: &str,
        client_state: &str,
        lifetime: std::time::Duration,
    ) -> Result<Subscription, GraphError> {
        let expiration = crate::time::to_iso(crate::time::now() + lifetime.as_millis() as i64);
        self.post(
            "subscriptions",
            &json!({
                "changeType": "created,updated,deleted",
                "notificationUrl": notification_url,
                "lifecycleNotificationUrl": lifecycle_url,
                "resource": format!("/users/{}/chats/getAllMessages", segment(me)),
                "expirationDateTime": expiration,
                "clientState": client_state,
                "includeResourceData": false,
            }),
        )
        .await
    }

    pub async fn renew_subscription(
        &self,
        id: &str,
        lifetime: std::time::Duration,
    ) -> Result<(), GraphError> {
        let expiration = crate::time::to_iso(crate::time::now() + lifetime.as_millis() as i64);
        self.patch(
            &format!("subscriptions/{}", segment(id)),
            &json!({ "expirationDateTime": expiration }),
        )
        .await
    }

    pub async fn reauthorize_subscription(&self, id: &str) -> Result<(), GraphError> {
        self.post_no_content(
            &format!("subscriptions/{}/reauthorize", segment(id)),
            &json!({}),
        )
        .await
    }

    /// `POST /subscriptions` for the person's own presence, without resource
    /// data (Presence.Read.All). Graph allows at most an hour.
    pub async fn subscribe_presence(
        &self,
        me: &str,
        notification_url: &str,
        lifecycle_url: &str,
        client_state: &str,
        lifetime: std::time::Duration,
    ) -> Result<Subscription, GraphError> {
        let lifetime = lifetime.min(std::time::Duration::from_secs(60 * 60));
        let expiration = crate::time::to_iso(crate::time::now() + lifetime.as_millis() as i64);
        self.post(
            "subscriptions",
            &json!({
                "changeType": "updated",
                "notificationUrl": notification_url,
                "lifecycleNotificationUrl": lifecycle_url,
                "resource": format!("communications/presences/{}", segment(me)),
                "expirationDateTime": expiration,
                "clientState": client_state,
                "includeResourceData": false,
            }),
        )
        .await
    }

    /// The chat of the meeting with this join link:
    /// `GET /me/onlineMeetings?$filter=JoinWebUrl eq '…'` (OnlineMeetings.Read;
    /// organisers and invited attendees), then the chat itself.
    pub async fn meeting_chat(&self, join_url: &str) -> Result<Option<Chat>, GraphError> {
        let meetings: g::Page<g::OnlineMeeting> = self
            .get(&format!(
                "me/onlineMeetings?$filter=JoinWebUrl%20eq%20'{}'",
                urlencoding::encode(join_url)
            ))
            .await?;
        let Some(thread) = meetings
            .value
            .into_iter()
            .find_map(|meeting| meeting.chat_info.and_then(|info| info.thread_id))
            .filter(|thread| !thread.is_empty())
        else {
            return Ok(None);
        };
        self.chat(&thread).await.map(Some)
    }

    pub async fn delete_subscription(&self, id: &str) -> Result<(), GraphError> {
        self.delete(&format!("subscriptions/{}", segment(id))).await
    }
}

fn message_path(conversation: &ConversationId, message: &str) -> String {
    match conversation {
        ConversationId::Chat(chat) => {
            format!("chats/{}/messages/{}", segment(chat), segment(message))
        }
        ConversationId::Channel { team, channel } => format!(
            "teams/{}/channels/{}/messages/{}",
            segment(team),
            segment(channel),
            segment(message)
        ),
    }
}

fn search_hit(hit: g::SearchHit) -> Option<SearchHit> {
    let resource = hit.resource?;
    let chat = resource.get("chatId").and_then(|v| v.as_str());
    let team = resource
        .pointer("/channelIdentity/teamId")
        .and_then(|v| v.as_str());
    let channel = resource
        .pointer("/channelIdentity/channelId")
        .and_then(|v| v.as_str());
    let conversation = match (chat, team, channel) {
        (Some(chat), _, _) => ConversationId::Chat(chat.to_owned()),
        (None, Some(team), Some(channel)) => ConversationId::Channel {
            team: team.to_owned(),
            channel: channel.to_owned(),
        },
        _ => return None,
    };
    Some(SearchHit {
        conversation,
        message_id: resource.get("id").and_then(|v| v.as_str())?.to_owned(),
        author: resource
            .pointer("/from/emailAddress/name")
            .or_else(|| resource.pointer("/from/user/displayName"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_owned(),
        snippet: crate::html::parse(hit.summary.as_deref().unwrap_or_default()).summary(200),
        at: resource
            .get("createdDateTime")
            .and_then(|v| v.as_str())
            .and_then(crate::time::parse_iso)
            .unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::super::testing;
    use super::*;
    use wiremock::matchers::{body_json, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn graph(server: &MockServer) -> Graph {
        Graph::with_base(
            reqwest::Client::new(),
            testing::session("at", &server.uri()).await,
            &server.uri(),
        )
    }

    #[tokio::test]
    async fn chats_follow_next_links() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/me/chats"))
            .and(query_param("$top", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": [{"id": "19:a@thread.v2", "chatType": "oneOnOne"}],
                "@odata.nextLink": format!("{}/me/chats?$skiptoken=page2", server.uri())
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/me/chats"))
            .and(query_param("$skiptoken", "page2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": [{"id": "19:b@thread.v2", "chatType": "group", "topic": "Launch"}]
            })))
            .mount(&server)
            .await;
        let chats = graph(&server).await.chats(3).await.unwrap();
        assert_eq!(chats.len(), 2);
        assert_eq!(chats[1].topic.as_deref(), Some("Launch"));
    }

    #[tokio::test]
    async fn chats_fall_back_to_two_listings() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/me/chats"))
            .and(query_param("$expand", "members,lastMessagePreview"))
            .respond_with(
                ResponseTemplate::new(400).set_body_json(json!({"error": {"code": "BadRequest"}})),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/me/chats"))
            .and(query_param("$expand", "lastMessagePreview"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"value": [{
                "id": "19:a@unq.gbl.spaces", "chatType": "oneOnOne",
                "lastMessagePreview": {"id": "1", "createdDateTime": "2026-10-11T10:00:00Z",
                    "body": {"contentType": "text", "content": "hi"}, "from": {"user": {"id": "u-ada", "displayName": "Ada"}}}
            }]})))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/me/chats"))
            .and(query_param("$expand", "members"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"value": [{
                "id": "19:a@unq.gbl.spaces", "chatType": "oneOnOne",
                "members": [{"userId": "u-ada", "displayName": "Ada"}, {"userId": "u-me", "displayName": "Me"}]
            }]})))
            .mount(&server)
            .await;
        let chats = graph(&server).await.chats(1).await.unwrap();
        assert_eq!(chats.len(), 1);
        assert_eq!(chats[0].title("u-me"), "Ada");
        assert_eq!(chats[0].preview.as_ref().unwrap().text, "hi");
    }

    #[tokio::test]
    async fn sending_posts_html_and_returns_the_message() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chats/19:a@thread.v2/messages"))
            .and(body_json(
                json!({"body": {"contentType": "html", "content": "<p>hi</p>"}}),
            ))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "id": "1702", "messageType": "message", "createdDateTime": "2026-10-11T10:00:00Z",
                "from": {"user": {"id": "u-me", "displayName": "Me"}},
                "body": {"contentType": "html", "content": "<p>hi</p>"}
            })))
            .expect(1)
            .mount(&server)
            .await;
        let message = graph(&server)
            .await
            .send_chat_message("19:a@thread.v2", &Outgoing::html("<p>hi</p>"))
            .await
            .unwrap();
        assert_eq!(message.id, "1702");
        assert_eq!(message.body.to_plain(), "hi");
    }

    #[test]
    fn outgoing_messages_carry_mentions_and_files() {
        let message = Outgoing {
            html: "<p><at id=\"0\">Ada</at> see this</p>".into(),
            mentions: vec![crate::model::MentionRef {
                user_id: "u-ada".into(),
                name: "Ada".into(),
            }],
            files: vec![crate::graph::files::SharedFile {
                id: "153fa47d-18c9-4179-be08-9879815a9f90".into(),
                name: "plan.pdf".into(),
                url: "https://contoso-my.sharepoint.com/personal/me/Documents/plan.pdf".into(),
            }],
        };
        assert_eq!(
            message.body(),
            json!({
                "body": {"contentType": "html", "content": "<p><at id=\"0\">Ada</at> see this</p><attachment id=\"153fa47d-18c9-4179-be08-9879815a9f90\"></attachment>"},
                "mentions": [{"id": 0, "mentionText": "Ada", "mentioned": {"user": {"id": "u-ada", "displayName": "Ada", "userIdentityType": "aadUser"}}}],
                "attachments": [{"id": "153fa47d-18c9-4179-be08-9879815a9f90", "contentType": "reference",
                    "contentUrl": "https://contoso-my.sharepoint.com/personal/me/Documents/plan.pdf", "name": "plan.pdf"}]
            })
        );
        assert_eq!(
            Outgoing::html("<p>x</p>").body(),
            json!({"body": {"contentType": "html", "content": "<p>x</p>"}})
        );
    }

    #[tokio::test]
    async fn quoting_uses_reply_with_quote() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chats/19:a@thread.v2/messages/replyWithQuote"))
            .and(body_json(json!({"messageIds": ["1690"], "replyMessage": {"body": {"contentType": "html", "content": "<p>yes</p>"}}})))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({"id": "1703", "messageType": "message"})))
            .expect(1)
            .mount(&server)
            .await;
        graph(&server)
            .await
            .reply_with_quote("19:a@thread.v2", "1690", &Outgoing::html("<p>yes</p>"))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn reactions_edits_deletes_and_read_marks_hit_their_endpoints() {
        let server = MockServer::start().await;
        for (verb, route) in [
            ("POST", "/chats/19:a@thread.v2/messages/1/setReaction"),
            ("POST", "/chats/19:a@thread.v2/messages/1/unsetReaction"),
            ("PATCH", "/chats/19:a@thread.v2/messages/1"),
            (
                "POST",
                "/users/u-me/chats/19:a@thread.v2/messages/1/softDelete",
            ),
            ("POST", "/chats/19:a@thread.v2/markChatReadForUser"),
        ] {
            Mock::given(method(verb))
                .and(path(route))
                .respond_with(ResponseTemplate::new(204))
                .expect(1)
                .mount(&server)
                .await;
        }
        let graph = graph(&server).await;
        let chat = ConversationId::Chat("19:a@thread.v2".into());
        graph.react(&chat, "1", "👍", true).await.unwrap();
        graph.react(&chat, "1", "👍", false).await.unwrap();
        graph
            .edit_message(&chat, "1", "<p>fixed</p>")
            .await
            .unwrap();
        graph.delete_message("u-me", &chat, "1").await.unwrap();
        graph
            .mark_chat_read("19:a@thread.v2", "u-me", "t1")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn changes_are_date_ranged() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/chats/19:a@thread.v2/messages"))
            .and(query_param(
                "$filter",
                "lastModifiedDateTime gt 2026-10-11T10:00:00.000Z",
            ))
            .and(query_param("$orderby", "lastModifiedDateTime desc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"value": []})))
            .expect(1)
            .mount(&server)
            .await;
        let since = crate::time::parse_iso("2026-10-11T10:00:00Z").unwrap();
        graph(&server)
            .await
            .chat_changes("19:a@thread.v2", since)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn missing_photos_are_none() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/users/u-x/photos/96x96/$value"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(json!({"error": {"code": "ImageNotFound"}})),
            )
            .mount(&server)
            .await;
        assert_eq!(graph(&server).await.photo("u-x").await.unwrap(), None);
    }

    #[tokio::test]
    async fn search_hits_name_their_conversation() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/search/query"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "value": [{"hitsContainers": [{"hits": [{
                    "hitId": "h1", "summary": "the <c0>launch</c0> plan",
                    "resource": {"@odata.type": "#microsoft.graph.chatMessage", "id": "1704", "chatId": "19:a@thread.v2",
                        "createdDateTime": "2026-10-11T10:00:00Z", "from": {"emailAddress": {"name": "Ada"}}}
                }]}]}]
            })))
            .mount(&server)
            .await;
        let hits = graph(&server)
            .await
            .search_messages("launch")
            .await
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].snippet, "the launch plan");
        assert_eq!(hits[0].author, "Ada");
    }
}
