//! Graph's shapes into Wren's model.

use super::types as g;
use crate::html;
use crate::model::{
    Attachment, AttachmentKind, Availability, Channel, ChannelKind, Chat, ChatKind, ConversationId,
    Meeting, Member, Message, Presence, Preview, Profile, Reaction, Team, reaction_emoji,
};
use crate::time::{parse_civil, parse_iso};

pub fn profile(user: g::User, tenant_id: Option<String>) -> Profile {
    Profile {
        display_name: user.display_name.unwrap_or_default(),
        user_principal_name: user.user_principal_name.unwrap_or_default(),
        mail: user.mail,
        job_title: user.job_title,
        id: user.id,
        tenant_id,
    }
}

fn body_text(body: Option<&g::ItemBody>) -> html::RichText {
    let Some(body) = body else {
        return html::RichText::default();
    };
    let content = body.content.as_deref().unwrap_or_default();
    match body.content_type.as_deref() {
        Some(kind) if kind.eq_ignore_ascii_case("text") => html::RichText::plain(content),
        _ => html::parse(content),
    }
}

pub fn chat(chat: g::Chat) -> Chat {
    let kind = match chat.chat_type.as_deref() {
        Some("group") => ChatKind::Group,
        Some("meeting") => ChatKind::Meeting,
        _ => ChatKind::OneOnOne,
    };
    let members = chat
        .members
        .unwrap_or_default()
        .into_iter()
        .filter_map(|member| {
            Some(Member {
                user_id: member.user_id?,
                display_name: member.display_name.unwrap_or_default(),
                email: member.email,
            })
        })
        .collect();
    let preview = chat.last_message_preview.map(|preview| {
        let deleted = preview.is_deleted.unwrap_or(false);
        let text = if deleted {
            "This message was deleted".to_owned()
        } else if preview
            .message_type
            .as_deref()
            .is_some_and(|t| t != "message")
        {
            system_summary(preview.event_detail.as_ref())
        } else {
            body_text(preview.body.as_ref()).summary(140)
        };
        Preview {
            message_id: preview.id,
            author_id: preview
                .from
                .as_ref()
                .and_then(|f| f.id())
                .map(str::to_owned),
            author: preview
                .from
                .as_ref()
                .and_then(|f| f.name())
                .map(str::to_owned),
            text,
            at: preview
                .created_date_time
                .as_deref()
                .and_then(parse_iso)
                .unwrap_or_default(),
            deleted,
        }
    });
    let viewpoint = chat.viewpoint.unwrap_or_default();
    Chat {
        id: chat.id,
        kind,
        topic: chat.topic,
        members,
        preview,
        last_read: viewpoint
            .last_message_read_date_time
            .as_deref()
            .and_then(parse_iso),
        updated: chat
            .last_updated_date_time
            .or(chat.created_date_time)
            .as_deref()
            .and_then(parse_iso)
            .unwrap_or_default(),
        web_url: chat.web_url,
        join_url: chat.online_meeting_info.and_then(|info| info.join_web_url),
        hidden: viewpoint.is_hidden.unwrap_or(false),
    }
}

/// Words for a system event, from its `eventDetail` type.
fn system_summary(detail: Option<&serde_json::Value>) -> String {
    let Some(detail) = detail else {
        return "Chat event".into();
    };
    let kind = detail
        .get("@odata.type")
        .and_then(|t| t.as_str())
        .unwrap_or_default()
        .trim_start_matches("#microsoft.graph.");
    let initiator = detail
        .pointer("/initiator/user/displayName")
        .and_then(|n| n.as_str())
        .filter(|n| !n.is_empty());
    let who = initiator.unwrap_or("Someone");
    let members = || {
        detail
            .get("members")
            .and_then(|m| m.as_array())
            .map(|members| {
                members
                    .iter()
                    .filter_map(|m| m.get("displayName").and_then(|n| n.as_str()))
                    .filter(|n| !n.is_empty())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .filter(|names| !names.is_empty())
            .unwrap_or_else(|| "people".into())
    };
    match kind {
        "membersAddedEventMessageDetail" => format!("{who} added {}", members()),
        "membersDeletedEventMessageDetail" => format!("{who} removed {}", members()),
        "membersJoinedEventMessageDetail" => format!("{} joined", members()),
        "membersLeftEventMessageDetail" => format!("{} left", members()),
        "chatRenamedEventMessageDetail" => {
            let name = detail
                .get("chatDisplayName")
                .and_then(|n| n.as_str())
                .unwrap_or_default();
            format!("{who} renamed the chat to “{name}”")
        }
        "callStartedEventMessageDetail" => "Call started".into(),
        "callEndedEventMessageDetail" => "Call ended".into(),
        "callRecordingEventMessageDetail" => "Call recorded".into(),
        "callTranscriptEventMessageDetail" => "Transcript available".into(),
        "meetingPolicyUpdatedEventMessageDetail" => "Meeting settings changed".into(),
        "messagePinnedEventMessageDetail" => format!("{who} pinned a message"),
        "messageUnpinnedEventMessageDetail" => format!("{who} unpinned a message"),
        "teamRenamedEventMessageDetail" | "channelRenamedEventMessageDetail" => {
            format!("{who} renamed the conversation")
        }
        "channelAddedEventMessageDetail" => format!("{who} added a channel"),
        "tabUpdatedEventMessageDetail" => format!("{who} updated a tab"),
        _ => "Chat event".into(),
    }
}

pub fn message(message: g::ChatMessage, conversation: Option<&ConversationId>) -> Message {
    let system = message
        .message_type
        .as_deref()
        .is_some_and(|kind| kind != "message");
    let deleted = message.deleted_date_time.is_some();
    let body = if system {
        html::RichText::plain(&system_summary(message.event_detail.as_ref()))
    } else if deleted {
        html::RichText::default()
    } else {
        body_text(message.body.as_ref())
    };
    let mut reactions: Vec<Reaction> = Vec::new();
    for reaction in &message.reactions {
        let emoji = reaction_emoji(&reaction.reaction_type);
        let user = reaction
            .user
            .as_ref()
            .and_then(|u| u.id())
            .unwrap_or_default()
            .to_owned();
        match reactions.iter_mut().find(|r| r.emoji == emoji) {
            Some(existing) => existing.users.push(user),
            None => reactions.push(Reaction {
                emoji,
                users: vec![user],
            }),
        }
    }
    let attachments = message.attachments.iter().filter_map(attachment).collect();
    let conversation = conversation.cloned().or_else(|| {
        if let Some(chat) = &message.chat_id {
            return Some(ConversationId::Chat(chat.clone()));
        }
        let identity = message.channel_identity.as_ref()?;
        Some(ConversationId::Channel {
            team: identity.team_id.clone()?,
            channel: identity.channel_id.clone()?,
        })
    });
    let created = message
        .created_date_time
        .as_deref()
        .and_then(parse_iso)
        .unwrap_or_default();
    let replies = message
        .replies
        .unwrap_or_default()
        .into_iter()
        .map(|reply| self::message(reply, conversation.as_ref()))
        .collect::<Vec<_>>();
    let mut replies = replies;
    replies.sort_by_key(|reply| reply.created);
    Message {
        author_id: message
            .from
            .as_ref()
            .and_then(|f| f.id())
            .map(str::to_owned),
        author: message
            .from
            .as_ref()
            .and_then(|f| f.name())
            .unwrap_or(if system { "" } else { "Unknown" })
            .to_owned(),
        created,
        modified: message
            .last_modified_date_time
            .as_deref()
            .and_then(parse_iso)
            .unwrap_or(created),
        edited: message.last_edited_date_time.is_some(),
        deleted,
        system,
        subject: message.subject.filter(|s| !s.trim().is_empty()),
        importance_high: matches!(message.importance.as_deref(), Some("high" | "urgent")),
        body,
        reactions,
        attachments,
        replies,
        id: message.id,
        reply_to: message.reply_to_id,
        conversation,
        delivery: Default::default(),
    }
}

fn attachment(attachment: &g::Attachment) -> Option<Attachment> {
    let content_type = attachment.content_type.clone();
    let kind = match content_type.as_deref() {
        Some("messageReference") => {
            let content: serde_json::Value = attachment
                .content
                .as_deref()
                .and_then(|c| serde_json::from_str(c).ok())
                .unwrap_or_default();
            AttachmentKind::Quote {
                message_id: content
                    .get("messageId")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned)
                    .or_else(|| attachment.id.clone())
                    .unwrap_or_default(),
                author: content
                    .pointer("/messageSender/user/displayName")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned),
                preview: content
                    .get("messagePreview")
                    .and_then(|v| v.as_str())
                    .map(|text| html::parse(text).summary(200))
                    .unwrap_or_default(),
            }
        }
        Some("reference") => AttachmentKind::File,
        Some(kind) if kind.starts_with("application/vnd.microsoft.card") => AttachmentKind::Card,
        Some("application/vnd.microsoft.teams.messaging-announcementBanner") => return None,
        Some(kind) if kind.contains('/') => AttachmentKind::File,
        _ => AttachmentKind::Other,
    };
    Some(Attachment {
        id: attachment.id.clone().unwrap_or_default(),
        kind,
        name: attachment.name.clone(),
        content_type,
        url: attachment.content_url.clone(),
    })
}

pub fn team(team: g::Team) -> Team {
    Team {
        name: team.display_name.unwrap_or_else(|| "Untitled team".into()),
        description: team.description.filter(|d| !d.trim().is_empty()),
        web_url: team.web_url,
        id: team.id,
    }
}

pub fn channel(channel: g::Channel, team_id: &str) -> Channel {
    Channel {
        name: channel
            .display_name
            .unwrap_or_else(|| "Untitled channel".into()),
        description: channel.description.filter(|d| !d.trim().is_empty()),
        kind: match channel.membership_type.as_deref() {
            Some("private") => ChannelKind::Private,
            Some("shared") => ChannelKind::Shared,
            _ => ChannelKind::Standard,
        },
        web_url: channel.web_url,
        team_id: team_id.to_owned(),
        id: channel.id,
    }
}

pub fn presence(presence: g::Presence) -> (String, Presence) {
    (
        presence.id,
        Presence {
            availability: Availability::from_graph(
                presence.availability.as_deref().unwrap_or_default(),
            ),
            activity: presence.activity,
            status_message: presence
                .status_message
                .and_then(|s| s.message)
                .and_then(|m| m.content)
                .map(|c| html::parse(&c).summary(120))
                .filter(|c| !c.is_empty()),
        },
    )
}

pub fn meeting(event: g::Event) -> Option<Meeting> {
    let start = event.start.as_ref()?;
    let end = event.end.as_ref()?;
    let join_url = event
        .online_meeting
        .and_then(|m| m.join_url)
        .or(event.online_meeting_url)
        .filter(|url| crate::calling::is_teams_link(url));
    Some(Meeting {
        subject: event
            .subject
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "(No title)".into()),
        start: parse_civil(&start.date_time, start.time_zone.as_deref())?,
        end: parse_civil(&end.date_time, end.time_zone.as_deref())?,
        all_day: event.is_all_day.unwrap_or(false),
        location: event
            .location
            .and_then(|l| l.display_name)
            .filter(|l| !l.trim().is_empty()),
        organizer: event
            .organizer
            .and_then(|o| o.email_address)
            .and_then(|e| e.name.or(e.address)),
        join_url,
        web_link: event.web_link,
        cancelled: event.is_cancelled.unwrap_or(false),
        preview: event.body_preview.filter(|p| !p.trim().is_empty()),
        id: event.id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_with_preview_members_and_viewpoint() {
        let raw: g::Chat = serde_json::from_value(serde_json::json!({
            "id": "19:abc@thread.v2",
            "topic": null,
            "chatType": "oneOnOne",
            "lastUpdatedDateTime": "2026-10-10T09:00:00Z",
            "webUrl": "https://teams.microsoft.com/l/chat/19%3Aabc%40thread.v2/0",
            "members": [
                {"@odata.type": "#microsoft.graph.aadUserConversationMember", "id": "m1", "displayName": "Ada", "userId": "u-ada", "email": "ada@contoso.com"},
                {"@odata.type": "#microsoft.graph.aadUserConversationMember", "id": "m2", "displayName": "Me", "userId": "u-me", "email": "me@contoso.com"}
            ],
            "lastMessagePreview": {
                "id": "1700", "createdDateTime": "2026-10-10T10:00:00Z", "isDeleted": false, "messageType": "message",
                "body": {"contentType": "html", "content": "<p>Ship <b>it</b></p>"},
                "from": {"user": {"id": "u-ada", "displayName": "Ada"}}
            },
            "viewpoint": {"isHidden": false, "lastMessageReadDateTime": "2026-10-10T09:30:00Z"}
        }))
        .unwrap();
        let chat = chat(raw);
        assert_eq!(chat.title("u-me"), "Ada");
        assert_eq!(chat.preview.as_ref().unwrap().text, "Ship it");
        assert!(chat.is_unread("u-me"));
        assert_eq!(
            chat.peer("u-me").unwrap().email.as_deref(),
            Some("ada@contoso.com")
        );
    }

    #[test]
    fn message_with_reactions_quote_and_edit() {
        let raw: g::ChatMessage = serde_json::from_value(serde_json::json!({
            "id": "1701", "replyToId": null, "messageType": "message",
            "createdDateTime": "2026-10-10T10:00:00Z", "lastModifiedDateTime": "2026-10-10T10:05:00Z",
            "lastEditedDateTime": "2026-10-10T10:04:00Z", "deletedDateTime": null,
            "chatId": "19:abc@thread.v2", "importance": "high",
            "from": {"user": {"id": "u-ada", "displayName": "Ada", "userIdentityType": "aadUser"}},
            "body": {"contentType": "html", "content": "<attachment id=\"1690\"></attachment><p>Agreed</p>"},
            "attachments": [{"id": "1690", "contentType": "messageReference", "content": "{\"messageId\":\"1690\",\"messagePreview\":\"Can we ship Friday?\",\"messageSender\":{\"user\":{\"id\":\"u-me\",\"displayName\":\"Me\"}}}"}],
            "reactions": [
                {"reactionType": "like", "createdDateTime": "2026-10-10T10:01:00Z", "user": {"user": {"id": "u-me"}}},
                {"reactionType": "👍", "createdDateTime": "2026-10-10T10:02:00Z", "user": {"user": {"id": "u-grace"}}},
                {"reactionType": "🎉", "user": {"user": {"id": "u-me"}}}
            ]
        }))
        .unwrap();
        let message = message(raw, None);
        assert_eq!(
            message.conversation,
            Some(ConversationId::Chat("19:abc@thread.v2".into()))
        );
        assert!(message.edited && message.importance_high);
        assert_eq!(message.body.to_plain(), "Agreed");
        assert_eq!(message.reactions.len(), 2);
        assert_eq!(message.reactions[0].users, vec!["u-me", "u-grace"]);
        let Some(Attachment {
            kind: AttachmentKind::Quote {
                preview, author, ..
            },
            ..
        }) = message.quote()
        else {
            panic!("quote expected");
        };
        assert_eq!(preview, "Can we ship Friday?");
        assert_eq!(author.as_deref(), Some("Me"));
    }

    #[test]
    fn system_events_get_words() {
        let raw: g::ChatMessage = serde_json::from_value(serde_json::json!({
            "id": "1", "messageType": "systemEventMessage", "createdDateTime": "2026-10-10T10:00:00Z",
            "body": {"contentType": "html", "content": "<systemEventMessage/>"},
            "eventDetail": {
                "@odata.type": "#microsoft.graph.membersAddedEventMessageDetail",
                "members": [{"id": "u-g", "displayName": "Grace"}],
                "initiator": {"user": {"id": "u-ada", "displayName": "Ada"}}
            }
        }))
        .unwrap();
        let message = message(raw, None);
        assert!(message.system);
        assert_eq!(message.body.to_plain(), "Ada added Grace");
    }

    #[test]
    fn deleted_messages_have_no_body() {
        let raw: g::ChatMessage = serde_json::from_value(serde_json::json!({
            "id": "1", "messageType": "message", "deletedDateTime": "2026-10-10T10:00:00Z",
            "body": {"contentType": "html", "content": "<p>secret</p>"}
        }))
        .unwrap();
        let message = message(raw, None);
        assert!(message.deleted);
        assert!(message.body.is_empty());
    }

    #[test]
    fn meetings_keep_only_teams_join_links() {
        let raw: g::Event = serde_json::from_value(serde_json::json!({
            "id": "e1", "subject": "Standup",
            "start": {"dateTime": "2026-10-12T09:00:00.0000000", "timeZone": "UTC"},
            "end": {"dateTime": "2026-10-12T09:15:00.0000000", "timeZone": "UTC"},
            "isOnlineMeeting": true,
            "onlineMeeting": {"joinUrl": "https://teams.microsoft.com/l/meetup-join/19%3ameeting_x%40thread.v2/0"}
        }))
        .unwrap();
        let meeting = meeting(raw).unwrap();
        assert!(meeting.join_url.is_some());
        assert_eq!(meeting.end - meeting.start, 15 * 60_000);

        let phishy: g::Event = serde_json::from_value(serde_json::json!({
            "id": "e2", "start": {"dateTime": "2026-10-12T09:00:00", "timeZone": "UTC"},
            "end": {"dateTime": "2026-10-12T10:00:00", "timeZone": "UTC"},
            "onlineMeeting": {"joinUrl": "https://evil.example/l/meetup-join/x"}
        }))
        .unwrap();
        assert_eq!(meeting_of(phishy).join_url, None);
    }

    fn meeting_of(event: g::Event) -> Meeting {
        meeting(event).unwrap()
    }
}
