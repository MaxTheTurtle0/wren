//! The local cache: one SQLite database per account.
//!
//! Chats, teams, channels, messages, meetings and photos are kept so Wren
//! opens instantly and navigates without waiting for the network. Rows hold
//! the model types as JSON next to the columns queries need, so adding a
//! model field needs no migration. Message text is indexed with FTS5 for
//! instant local search.
//!
//! The cache holds the person's work messages: the file is created private
//! (0600 on Unix) in a private folder. Encrypting it with a key from the OS
//! credential store is on the roadmap (docs/roadmap.md).

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::model::{Channel, Chat, ConversationId, Meeting, Message, SearchHit, Team};

const SCHEMA_VERSION: i32 = 1;

pub struct Store {
    db: Connection,
}

impl Store {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        if let Some(dir) = path.parent() {
            let _ = crate::paths::create_private_dir(dir);
        }
        let db = Connection::open(path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
        }
        Self::setup(db)
    }

    pub fn in_memory() -> rusqlite::Result<Self> {
        Self::setup(Connection::open_in_memory()?)
    }

    fn setup(db: Connection) -> rusqlite::Result<Self> {
        db.pragma_update(None, "journal_mode", "WAL")?;
        db.pragma_update(None, "synchronous", "NORMAL")?;
        db.pragma_update(None, "foreign_keys", "ON")?;
        db.busy_timeout(std::time::Duration::from_secs(5))?;
        let version: i32 = db.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version > SCHEMA_VERSION {
            // A newer Wren wrote this cache. It is only a cache: start over.
            log::warn!("cache schema {version} is newer than {SCHEMA_VERSION}; rebuilding it");
            db.execute_batch(
                "DROP TABLE IF EXISTS chats; DROP TABLE IF EXISTS teams; DROP TABLE IF EXISTS channels;
                 DROP TABLE IF EXISTS messages; DROP TABLE IF EXISTS messages_fts; DROP TABLE IF EXISTS meetings;
                 DROP TABLE IF EXISTS photos; DROP TABLE IF EXISTS meta;",
            )?;
            db.pragma_update(None, "user_version", 0)?;
        }
        let store = Self { db };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> rusqlite::Result<()> {
        let version: i32 = self
            .db
            .pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version < 1 {
            self.db.execute_batch(
                "BEGIN;
                CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
                CREATE TABLE IF NOT EXISTS chats (
                    id TEXT PRIMARY KEY,
                    activity INTEGER NOT NULL,
                    json TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS teams (id TEXT PRIMARY KEY, json TEXT NOT NULL);
                CREATE TABLE IF NOT EXISTS channels (
                    team TEXT NOT NULL,
                    id TEXT NOT NULL,
                    position INTEGER NOT NULL,
                    json TEXT NOT NULL,
                    PRIMARY KEY (team, id)
                );
                CREATE TABLE IF NOT EXISTS messages (
                    conversation TEXT NOT NULL,
                    id TEXT NOT NULL,
                    created INTEGER NOT NULL,
                    modified INTEGER NOT NULL,
                    json TEXT NOT NULL,
                    PRIMARY KEY (conversation, id)
                );
                CREATE INDEX IF NOT EXISTS messages_by_time ON messages (conversation, created);
                CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
                    body, author, conversation UNINDEXED, id UNINDEXED,
                    tokenize = 'unicode61 remove_diacritics 2'
                );
                CREATE TABLE IF NOT EXISTS meetings (id TEXT PRIMARY KEY, start INTEGER NOT NULL, json TEXT NOT NULL);
                CREATE TABLE IF NOT EXISTS photos (user TEXT PRIMARY KEY, fetched INTEGER NOT NULL, bytes BLOB);
                PRAGMA user_version = 1;
                COMMIT;",
            )?;
        }
        Ok(())
    }

    pub fn meta(&self, key: &str) -> Option<String> {
        self.db
            .query_row("SELECT value FROM meta WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .optional()
            .ok()
            .flatten()
    }

    pub fn set_meta(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.db.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    /// Replaces the chat list with Graph's answer. Chats Graph no longer
    /// lists keep their messages until the cache is cleared.
    pub fn save_chats(&mut self, chats: &[Chat]) -> rusqlite::Result<()> {
        let tx = self.db.transaction()?;
        tx.execute("DELETE FROM chats", [])?;
        {
            let mut insert =
                tx.prepare("INSERT INTO chats (id, activity, json) VALUES (?1, ?2, ?3)")?;
            for chat in chats {
                insert.execute(params![chat.id, chat.activity(), to_json(chat)])?;
            }
        }
        tx.commit()
    }

    pub fn upsert_chat(&self, chat: &Chat) -> rusqlite::Result<()> {
        self.db.execute(
            "INSERT INTO chats (id, activity, json) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET activity = excluded.activity, json = excluded.json",
            params![chat.id, chat.activity(), to_json(chat)],
        )?;
        Ok(())
    }

    pub fn chats(&self) -> rusqlite::Result<Vec<Chat>> {
        let mut query = self
            .db
            .prepare("SELECT json FROM chats ORDER BY activity DESC")?;
        let rows = query.query_map([], |row| row.get::<_, String>(0))?;
        Ok(rows.filter_map(|json| from_json(&json.ok()?)).collect())
    }

    pub fn save_teams(&mut self, teams: &[Team]) -> rusqlite::Result<()> {
        let tx = self.db.transaction()?;
        tx.execute("DELETE FROM teams", [])?;
        {
            let mut insert = tx.prepare("INSERT INTO teams (id, json) VALUES (?1, ?2)")?;
            for team in teams {
                insert.execute(params![team.id, to_json(team)])?;
            }
        }
        tx.commit()
    }

    pub fn teams(&self) -> rusqlite::Result<Vec<Team>> {
        let mut query = self.db.prepare("SELECT json FROM teams")?;
        let rows = query.query_map([], |row| row.get::<_, String>(0))?;
        let mut teams: Vec<Team> = rows.filter_map(|json| from_json(&json.ok()?)).collect();
        teams.sort_by_key(|team| team.name.to_lowercase());
        Ok(teams)
    }

    pub fn save_channels(&mut self, team: &str, channels: &[Channel]) -> rusqlite::Result<()> {
        let tx = self.db.transaction()?;
        tx.execute("DELETE FROM channels WHERE team = ?1", [team])?;
        {
            let mut insert = tx.prepare(
                "INSERT INTO channels (team, id, position, json) VALUES (?1, ?2, ?3, ?4)",
            )?;
            for (position, channel) in channels.iter().enumerate() {
                insert.execute(params![team, channel.id, position as i64, to_json(channel)])?;
            }
        }
        tx.commit()
    }

    pub fn channels(&self) -> rusqlite::Result<Vec<Channel>> {
        let mut query = self
            .db
            .prepare("SELECT json FROM channels ORDER BY team, position")?;
        let rows = query.query_map([], |row| row.get::<_, String>(0))?;
        Ok(rows.filter_map(|json| from_json(&json.ok()?)).collect())
    }

    /// Inserts or replaces messages and their search entries.
    pub fn save_messages(
        &mut self,
        conversation: &ConversationId,
        messages: &[Message],
    ) -> rusqlite::Result<()> {
        let key = conversation.key();
        let tx = self.db.transaction()?;
        {
            let mut insert = tx.prepare(
                "INSERT INTO messages (conversation, id, created, modified, json) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(conversation, id) DO UPDATE SET created = excluded.created,
                 modified = excluded.modified, json = excluded.json",
            )?;
            let mut forget =
                tx.prepare("DELETE FROM messages_fts WHERE conversation = ?1 AND id = ?2")?;
            let mut index = tx.prepare(
                "INSERT INTO messages_fts (body, author, conversation, id) VALUES (?1, ?2, ?3, ?4)",
            )?;
            for message in messages {
                insert.execute(params![
                    key,
                    message.id,
                    message.created,
                    message.modified,
                    to_json(message)
                ])?;
                forget.execute(params![key, message.id])?;
                if !message.system && !message.deleted {
                    let mut text = message.body.to_plain();
                    for reply in &message.replies {
                        text.push('\n');
                        text.push_str(&reply.body.to_plain());
                    }
                    if let Some(subject) = &message.subject {
                        text = format!("{subject}\n{text}");
                    }
                    index.execute(params![text, message.author, key, message.id])?;
                }
            }
        }
        tx.commit()
    }

    /// The newest `limit` messages of a conversation, oldest first.
    pub fn messages(
        &self,
        conversation: &ConversationId,
        limit: usize,
    ) -> rusqlite::Result<Vec<Message>> {
        let mut query = self.db.prepare(
            "SELECT json FROM (SELECT json, created FROM messages WHERE conversation = ?1
             ORDER BY created DESC LIMIT ?2) ORDER BY created ASC",
        )?;
        let rows = query.query_map(params![conversation.key(), limit as i64], |row| {
            row.get::<_, String>(0)
        })?;
        Ok(rows.filter_map(|json| from_json(&json.ok()?)).collect())
    }

    /// The newest change Wren has seen in a conversation, for date-ranged
    /// refreshes.
    pub fn last_modified(&self, conversation: &ConversationId) -> Option<i64> {
        self.db
            .query_row(
                "SELECT MAX(modified) FROM messages WHERE conversation = ?1",
                [conversation.key()],
                |row| row.get::<_, Option<i64>>(0),
            )
            .ok()
            .flatten()
    }

    /// When the oldest cached message of a conversation was written.
    pub fn oldest(&self, conversation: &ConversationId) -> Option<i64> {
        self.db
            .query_row(
                "SELECT MIN(created) FROM messages WHERE conversation = ?1",
                [conversation.key()],
                |row| row.get::<_, Option<i64>>(0),
            )
            .ok()
            .flatten()
    }

    pub fn delete_message(&self, conversation: &ConversationId, id: &str) -> rusqlite::Result<()> {
        let key = conversation.key();
        self.db.execute(
            "DELETE FROM messages WHERE conversation = ?1 AND id = ?2",
            params![key, id],
        )?;
        self.db.execute(
            "DELETE FROM messages_fts WHERE conversation = ?1 AND id = ?2",
            params![key, id],
        )?;
        Ok(())
    }

    /// Local full-text search, newest first. Each word must match a prefix.
    pub fn search(&self, query: &str, limit: usize) -> rusqlite::Result<Vec<SearchHit>> {
        let Some(expression) = fts_query(query) else {
            return Ok(Vec::new());
        };
        let mut statement = self.db.prepare(
            "SELECT f.conversation, f.id, f.author, snippet(messages_fts, 0, '', '', '…', 12), m.created
             FROM messages_fts f JOIN messages m ON m.conversation = f.conversation AND m.id = f.id
             WHERE messages_fts MATCH ?1 ORDER BY m.created DESC LIMIT ?2",
        )?;
        let rows = statement.query_map(params![expression, limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })?;
        Ok(rows
            .filter_map(Result::ok)
            .filter_map(|(conversation, id, author, snippet, at)| {
                Some(SearchHit {
                    conversation: ConversationId::from_key(&conversation)?,
                    message_id: id,
                    author,
                    snippet: snippet.replace('\n', " "),
                    at,
                })
            })
            .collect())
    }

    pub fn save_meetings(&mut self, meetings: &[Meeting]) -> rusqlite::Result<()> {
        let tx = self.db.transaction()?;
        tx.execute("DELETE FROM meetings", [])?;
        {
            let mut insert =
                tx.prepare("INSERT INTO meetings (id, start, json) VALUES (?1, ?2, ?3)")?;
            for meeting in meetings {
                insert.execute(params![meeting.id, meeting.start, to_json(meeting)])?;
            }
        }
        tx.commit()
    }

    pub fn meetings(&self) -> rusqlite::Result<Vec<Meeting>> {
        let mut query = self
            .db
            .prepare("SELECT json FROM meetings ORDER BY start")?;
        let rows = query.query_map([], |row| row.get::<_, String>(0))?;
        Ok(rows.filter_map(|json| from_json(&json.ok()?)).collect())
    }

    /// A cached photo: `Some(None)` means the person has none.
    pub fn photo(&self, user: &str, max_age_ms: i64) -> Option<Option<Vec<u8>>> {
        self.db
            .query_row(
                "SELECT bytes FROM photos WHERE user = ?1 AND fetched > ?2",
                params![user, crate::time::now() - max_age_ms],
                |row| row.get::<_, Option<Vec<u8>>>(0),
            )
            .optional()
            .ok()
            .flatten()
    }

    pub fn save_photo(&self, user: &str, bytes: Option<&[u8]>) -> rusqlite::Result<()> {
        self.db.execute(
            "INSERT INTO photos (user, fetched, bytes) VALUES (?1, ?2, ?3)
             ON CONFLICT(user) DO UPDATE SET fetched = excluded.fetched, bytes = excluded.bytes",
            params![user, crate::time::now(), bytes],
        )?;
        Ok(())
    }

    /// Counts for the performance overlay and diagnostics (no content).
    pub fn stats(&self) -> (i64, i64) {
        let count = |table: &str| {
            self.db
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap_or_default()
        };
        (count("chats"), count("messages"))
    }
}

fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

fn from_json<T: serde::de::DeserializeOwned>(json: &str) -> Option<T> {
    serde_json::from_str(json).ok()
}

/// Turns typed words into an FTS5 expression: each word quoted (so
/// operators in the text are literal) and matched as a prefix.
fn fts_query(text: &str) -> Option<String> {
    let words: Vec<String> = text
        .split_whitespace()
        .map(|word| word.replace('"', ""))
        .filter(|word| !word.is_empty())
        .map(|word| format!("\"{word}\"*"))
        .collect();
    (!words.is_empty()).then(|| words.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::RichText;
    use crate::model::Preview;

    fn message(id: &str, created: i64, text: &str) -> Message {
        Message {
            id: id.into(),
            author: "Ada".into(),
            author_id: Some("u-ada".into()),
            created,
            modified: created,
            body: RichText::plain(text),
            ..Default::default()
        }
    }

    #[test]
    fn chats_come_back_newest_first() {
        let mut store = Store::in_memory().unwrap();
        let chat = |id: &str, at: i64| Chat {
            id: id.into(),
            preview: Some(Preview {
                at,
                ..Default::default()
            }),
            ..Default::default()
        };
        store
            .save_chats(&[chat("old", 1), chat("new", 5), chat("mid", 3)])
            .unwrap();
        let ids: Vec<String> = store.chats().unwrap().into_iter().map(|c| c.id).collect();
        assert_eq!(ids, ["new", "mid", "old"]);
    }

    #[test]
    fn messages_page_and_update_in_place() {
        let mut store = Store::in_memory().unwrap();
        let conversation = ConversationId::Chat("19:a".into());
        let messages: Vec<Message> = (0..10)
            .map(|i| message(&i.to_string(), i, "hello"))
            .collect();
        store.save_messages(&conversation, &messages).unwrap();
        let newest = store.messages(&conversation, 3).unwrap();
        assert_eq!(
            newest.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
            ["7", "8", "9"]
        );
        let mut edited = message("9", 9, "edited text");
        edited.modified = 100;
        store.save_messages(&conversation, &[edited]).unwrap();
        assert_eq!(
            store.messages(&conversation, 1).unwrap()[0].body.to_plain(),
            "edited text"
        );
        assert_eq!(store.last_modified(&conversation), Some(100));
        assert_eq!(store.stats(), (0, 10));
    }

    #[test]
    fn search_finds_prefixes_and_ignores_operators() {
        let mut store = Store::in_memory().unwrap();
        let chat = ConversationId::Chat("19:a".into());
        let channel = ConversationId::Channel {
            team: "t".into(),
            channel: "19:c".into(),
        };
        store
            .save_messages(
                &chat,
                &[
                    message("1", 1, "The launch is on Friday"),
                    message("2", 2, "Lunch?"),
                ],
            )
            .unwrap();
        store
            .save_messages(&channel, &[message("3", 3, "Launch checklist ready")])
            .unwrap();
        let hits = store.search("laun", 10).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].conversation, channel);
        assert!(store.search("\"unbalanced AND OR (", 10).is_ok());
        assert!(store.search("   ", 10).unwrap().is_empty());
        // Diacritics fold.
        store
            .save_messages(&chat, &[message("4", 4, "Café opening")])
            .unwrap();
        assert_eq!(store.search("cafe", 10).unwrap().len(), 1);
        // Updated text replaces its index entry.
        store
            .save_messages(&chat, &[message("4", 4, "Tea opening")])
            .unwrap();
        assert!(store.search("cafe", 10).unwrap().is_empty());
    }

    #[test]
    fn files_reopen_with_their_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("accounts").join("a").join("wren.db");
        {
            let mut store = Store::open(&path).unwrap();
            store
                .save_teams(&[Team {
                    id: "t".into(),
                    name: "Design".into(),
                    ..Default::default()
                }])
                .unwrap();
            store.set_meta("k", "v").unwrap();
        }
        let store = Store::open(&path).unwrap();
        assert_eq!(store.teams().unwrap()[0].name, "Design");
        assert_eq!(store.meta("k").as_deref(), Some("v"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn photos_remember_absence() {
        let store = Store::in_memory().unwrap();
        assert_eq!(store.photo("u", 1000), None);
        store.save_photo("u", None).unwrap();
        assert_eq!(store.photo("u", 60_000), Some(None));
        store.save_photo("v", Some(b"jpeg")).unwrap();
        assert_eq!(store.photo("v", 60_000), Some(Some(b"jpeg".to_vec())));
    }
}
