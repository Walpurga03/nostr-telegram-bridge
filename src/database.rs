use rusqlite::{Connection, Result as SqlResult, params};
use std::path::Path;
use std::sync::Mutex;
use log::{info, debug};

/// Richtung der Nachricht
#[derive(Debug, Clone, PartialEq)]
pub enum MessageDirection {
    TelegramToNostr,
    NostrToTelegram,
}

impl MessageDirection {
    pub fn to_string(&self) -> &'static str {
        match self {
            MessageDirection::TelegramToNostr => "telegram_to_nostr",
            MessageDirection::NostrToTelegram => "nostr_to_telegram",
        }
    }

    #[allow(dead_code)]
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "telegram_to_nostr" => Some(MessageDirection::TelegramToNostr),
            "nostr_to_telegram" => Some(MessageDirection::NostrToTelegram),
            _ => None,
        }
    }
}

/// Mapping-Eintrag zwischen Telegram und Nostr
#[derive(Debug, Clone)]
pub struct MessageMapping {
    #[allow(dead_code)]
    pub id: Option<i64>,
    pub telegram_chat_id: i64,
    pub telegram_message_id: i64,
    pub nostr_event_id: String,
    pub nostr_recipient_pubkey: String,
    pub direction: MessageDirection,
    pub timestamp: i64,
}

/// Datenbank-Handler für die Bridge (thread-safe)
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Erstellt oder öffnet die Datenbank
    pub fn new<P: AsRef<Path>>(path: P) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        let db = Database { 
            conn: Mutex::new(conn) 
        };
        db.init_schema()?;
        info!("Datenbank initialisiert");
        Ok(db)
    }

    /// Erstellt das Datenbank-Schema
    fn init_schema(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS message_mapping (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                telegram_chat_id INTEGER NOT NULL,
                telegram_message_id INTEGER NOT NULL,
                nostr_event_id TEXT NOT NULL,
                nostr_recipient_pubkey TEXT NOT NULL,
                direction TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                UNIQUE(telegram_chat_id, telegram_message_id),
                UNIQUE(nostr_event_id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_telegram_lookup 
             ON message_mapping(telegram_chat_id, telegram_message_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_nostr_lookup 
             ON message_mapping(nostr_event_id)",
            [],
        )?;

        debug!("Datenbank-Schema erstellt");
        Ok(())
    }

    /// Speichert ein neues Mapping
    pub fn save_mapping(&self, mapping: &MessageMapping) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT INTO message_mapping 
             (telegram_chat_id, telegram_message_id, nostr_event_id, 
              nostr_recipient_pubkey, direction, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                mapping.telegram_chat_id,
                mapping.telegram_message_id,
                mapping.nostr_event_id,
                mapping.nostr_recipient_pubkey,
                mapping.direction.to_string(),
                mapping.timestamp,
            ],
        )?;

        let id = conn.last_insert_rowid();
        debug!("Mapping gespeichert: ID {}", id);
        Ok(id)
    }

    /// Prüft ob eine Telegram-Nachricht bereits verarbeitet wurde (Loop-Schutz)
    pub fn telegram_message_exists(&self, chat_id: i64, message_id: i64) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM message_mapping 
             WHERE telegram_chat_id = ?1 AND telegram_message_id = ?2"
        )?;

        let count: i64 = stmt.query_row(params![chat_id, message_id], |row| row.get(0))?;
        Ok(count > 0)
    }

    /// Prüft ob ein Nostr-Event bereits verarbeitet wurde (Loop-Schutz)
    pub fn nostr_event_exists(&self, event_id: &str) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM message_mapping WHERE nostr_event_id = ?1"
        )?;

        let count: i64 = stmt.query_row(params![event_id], |row| row.get(0))?;
        Ok(count > 0)
    }

    /// Findet das Nostr-Event zu einer Telegram-Nachricht (für Replys)
    #[allow(dead_code)]
    pub fn find_nostr_event_by_telegram(
        &self,
        chat_id: i64,
        message_id: i64,
    ) -> SqlResult<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT nostr_event_id FROM message_mapping 
             WHERE telegram_chat_id = ?1 AND telegram_message_id = ?2"
        )?;

        let result = stmt.query_row(params![chat_id, message_id], |row| {
            row.get::<_, String>(0)
        });

        match result {
            Ok(event_id) => Ok(Some(event_id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Findet die Telegram-Nachricht zu einem Nostr-Event (für Replys)
    #[allow(dead_code)]
    pub fn find_telegram_message_by_nostr(
        &self,
        event_id: &str,
    ) -> SqlResult<Option<(i64, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT telegram_chat_id, telegram_message_id FROM message_mapping 
             WHERE nostr_event_id = ?1"
        )?;

        let result = stmt.query_row(params![event_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        });

        match result {
            Ok(ids) => Ok(Some(ids)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Gibt Statistiken über die Datenbank zurück
    pub fn get_stats(&self) -> SqlResult<(i64, i64, i64)> {
        let conn = self.conn.lock().unwrap();
        
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM message_mapping",
            [],
            |row| row.get(0),
        )?;

        let telegram_to_nostr: i64 = conn.query_row(
            "SELECT COUNT(*) FROM message_mapping WHERE direction = 'telegram_to_nostr'",
            [],
            |row| row.get(0),
        )?;

        let nostr_to_telegram: i64 = conn.query_row(
            "SELECT COUNT(*) FROM message_mapping WHERE direction = 'nostr_to_telegram'",
            [],
            |row| row.get(0),
        )?;

        Ok((total, telegram_to_nostr, nostr_to_telegram))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_db() -> Database {
        Database::new(":memory:").expect("Failed to create test database")
    }

    fn get_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    #[test]
    fn test_database_creation() {
        let db = create_test_db();
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.0, 0); // Total should be 0
    }

    #[test]
    fn test_save_and_check_telegram_message() {
        let db = create_test_db();
        
        let mapping = MessageMapping {
            id: None,
            telegram_chat_id: -1001234567890,
            telegram_message_id: 123,
            nostr_event_id: "abc123".to_string(),
            nostr_recipient_pubkey: "npub1test".to_string(),
            direction: MessageDirection::TelegramToNostr,
            timestamp: get_timestamp(),
        };

        db.save_mapping(&mapping).unwrap();

        assert!(db.telegram_message_exists(-1001234567890, 123).unwrap());
        assert!(!db.telegram_message_exists(-1001234567890, 999).unwrap());
    }

    #[test]
    fn test_save_and_check_nostr_event() {
        let db = create_test_db();
        
        let mapping = MessageMapping {
            id: None,
            telegram_chat_id: -1001234567890,
            telegram_message_id: 456,
            nostr_event_id: "def456".to_string(),
            nostr_recipient_pubkey: "npub1test".to_string(),
            direction: MessageDirection::NostrToTelegram,
            timestamp: get_timestamp(),
        };

        db.save_mapping(&mapping).unwrap();

        assert!(db.nostr_event_exists("def456").unwrap());
        assert!(!db.nostr_event_exists("xyz999").unwrap());
    }

    #[test]
    fn test_find_mappings() {
        let db = create_test_db();
        
        let mapping = MessageMapping {
            id: None,
            telegram_chat_id: -1001234567890,
            telegram_message_id: 789,
            nostr_event_id: "ghi789".to_string(),
            nostr_recipient_pubkey: "npub1test".to_string(),
            direction: MessageDirection::TelegramToNostr,
            timestamp: get_timestamp(),
        };

        db.save_mapping(&mapping).unwrap();

        // Test Telegram -> Nostr lookup
        let nostr_event = db.find_nostr_event_by_telegram(-1001234567890, 789).unwrap();
        assert_eq!(nostr_event, Some("ghi789".to_string()));

        // Test Nostr -> Telegram lookup
        let telegram_msg = db.find_telegram_message_by_nostr("ghi789").unwrap();
        assert_eq!(telegram_msg, Some((-1001234567890, 789)));
    }

    #[test]
    fn test_duplicate_prevention() {
        let db = create_test_db();
        
        let mapping = MessageMapping {
            id: None,
            telegram_chat_id: -1001234567890,
            telegram_message_id: 111,
            nostr_event_id: "unique123".to_string(),
            nostr_recipient_pubkey: "npub1test".to_string(),
            direction: MessageDirection::TelegramToNostr,
            timestamp: get_timestamp(),
        };

        db.save_mapping(&mapping).unwrap();

        // Try to save duplicate - should fail
        let result = db.save_mapping(&mapping);
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics() {
        let db = create_test_db();
        
        // Add some mappings
        for i in 0..5 {
            let mapping = MessageMapping {
                id: None,
                telegram_chat_id: -1001234567890,
                telegram_message_id: i,
                nostr_event_id: format!("event{}", i),
                nostr_recipient_pubkey: "npub1test".to_string(),
                direction: if i % 2 == 0 {
                    MessageDirection::TelegramToNostr
                } else {
                    MessageDirection::NostrToTelegram
                },
                timestamp: get_timestamp(),
            };
            db.save_mapping(&mapping).unwrap();
        }

        let (total, t_to_n, n_to_t) = db.get_stats().unwrap();
        assert_eq!(total, 5);
        assert_eq!(t_to_n, 3); // 0, 2, 4
        assert_eq!(n_to_t, 2); // 1, 3
    }
}
