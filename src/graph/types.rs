//! Graph's JSON shapes, only as far as Wren reads them. Every field is
//! optional or defaulted: Graph adds fields freely and leaves many out.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Page<T> {
    #[serde(default = "Vec::new")]
    pub value: Vec<T>,
    #[serde(rename = "@odata.nextLink", default)]
    pub next_link: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct User {
    pub id: String,
    pub display_name: Option<String>,
    pub mail: Option<String>,
    pub user_principal_name: Option<String>,
    pub job_title: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Identity {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct IdentitySet {
    pub user: Option<Identity>,
    pub application: Option<Identity>,
    pub device: Option<Identity>,
}

impl IdentitySet {
    pub fn id(&self) -> Option<&str> {
        self.user.as_ref().and_then(|u| u.id.as_deref())
    }

    pub fn name(&self) -> Option<&str> {
        [&self.user, &self.application, &self.device]
            .into_iter()
            .flatten()
            .find_map(|identity| identity.display_name.as_deref())
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ItemBody {
    pub content_type: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConversationMember {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub user_id: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChatMessageInfo {
    pub id: String,
    pub created_date_time: Option<String>,
    pub is_deleted: Option<bool>,
    pub message_type: Option<String>,
    pub body: Option<ItemBody>,
    pub from: Option<IdentitySet>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChatViewpoint {
    pub is_hidden: Option<bool>,
    pub last_message_read_date_time: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OnlineMeetingInfo {
    pub join_web_url: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Chat {
    pub id: String,
    pub topic: Option<String>,
    pub chat_type: Option<String>,
    pub created_date_time: Option<String>,
    pub last_updated_date_time: Option<String>,
    pub web_url: Option<String>,
    pub members: Option<Vec<ConversationMember>>,
    pub last_message_preview: Option<ChatMessageInfo>,
    pub viewpoint: Option<ChatViewpoint>,
    pub online_meeting_info: Option<OnlineMeetingInfo>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Attachment {
    pub id: Option<String>,
    pub content_type: Option<String>,
    pub content_url: Option<String>,
    pub content: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Reaction {
    pub reaction_type: String,
    pub display_name: Option<String>,
    pub created_date_time: Option<String>,
    pub user: Option<IdentitySet>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChannelIdentity {
    pub team_id: Option<String>,
    pub channel_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ChatMessage {
    pub id: String,
    pub reply_to_id: Option<String>,
    pub message_type: Option<String>,
    pub created_date_time: Option<String>,
    pub last_modified_date_time: Option<String>,
    pub last_edited_date_time: Option<String>,
    pub deleted_date_time: Option<String>,
    pub subject: Option<String>,
    pub importance: Option<String>,
    pub body: Option<ItemBody>,
    pub from: Option<IdentitySet>,
    pub attachments: Vec<Attachment>,
    pub reactions: Vec<Reaction>,
    pub event_detail: Option<serde_json::Value>,
    pub chat_id: Option<String>,
    pub channel_identity: Option<ChannelIdentity>,
    pub replies: Option<Vec<ChatMessage>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Team {
    pub id: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub web_url: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Channel {
    pub id: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub membership_type: Option<String>,
    pub web_url: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StatusMessageBody {
    pub content: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StatusMessage {
    pub message: Option<StatusMessageBody>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Presence {
    pub id: String,
    pub availability: Option<String>,
    pub activity: Option<String>,
    pub status_message: Option<StatusMessage>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DateTimeTimeZone {
    pub date_time: String,
    pub time_zone: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Location {
    pub display_name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EmailAddress {
    pub name: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Recipient {
    pub email_address: Option<EmailAddress>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OnlineMeeting {
    pub join_url: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Event {
    pub id: String,
    pub subject: Option<String>,
    pub body_preview: Option<String>,
    pub start: Option<DateTimeTimeZone>,
    pub end: Option<DateTimeTimeZone>,
    pub is_all_day: Option<bool>,
    pub is_cancelled: Option<bool>,
    pub location: Option<Location>,
    pub organizer: Option<Recipient>,
    pub is_online_meeting: Option<bool>,
    pub online_meeting: Option<OnlineMeeting>,
    pub online_meeting_url: Option<String>,
    pub web_link: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ErrorBody {
    pub error: Option<ErrorDetail>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ErrorDetail {
    pub code: Option<String>,
    pub message: Option<String>,
}

// Microsoft Search.

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SearchResponse {
    pub value: Vec<SearchResultSet>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SearchResultSet {
    pub hits_containers: Vec<HitsContainer>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HitsContainer {
    pub hits: Vec<SearchHit>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SearchHit {
    pub hit_id: Option<String>,
    pub summary: Option<String>,
    pub resource: Option<serde_json::Value>,
}
