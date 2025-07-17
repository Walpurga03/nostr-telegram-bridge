use std::env;
use std::result::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Umgebungsvariable '{0}' fehlt")]
    MissingEnvVar(String),
    #[error("Ungültiger Wert für '{var}': {msg}")]
    InvalidValue { var: String, msg: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum EncryptionType {
    Nip04,
    Nip17,
    Public,
    Group,
}

impl EncryptionType {
    pub fn from_str(s: &str) -> Result<Self, ConfigError> {
        match s.to_lowercase().as_str() {
            "nip04" => Ok(EncryptionType::Nip04),
            "nip17" => Ok(EncryptionType::Nip17),
            "public" => Ok(EncryptionType::Public),
            "group" => Ok(EncryptionType::Group),
            _ => Err(ConfigError::InvalidValue {
                var: "ENCRYPTION_TYPE".to_string(),
                msg: "Muss 'nip04', 'nip17', 'public' oder 'group' sein".to_string(),
            }),
        }
    }
}

/// Konfiguration für die Bridge, geladen aus Umgebungsvariablen
#[derive(Debug, Clone)]
pub struct Config {
    /// Telegram Bot Token (vom BotFather)
    pub telegram_bot_token: String,
    /// Telegram Gruppen-ID (z.B. -1001234567890)
    pub telegram_group_id: i64,
    /// Nostr Private Key (nsec...)
    pub nostr_private_key: String,
    /// Nostr Empfänger-Pubkey (npub...) - Optional für public/group
    pub nostr_public_key: Option<String>,
    /// Liste von Nostr-Relays
    pub nostr_relays: Vec<String>,
    /// Verschlüsselungstyp
    pub encryption_type: EncryptionType,
    /// Nostr Gruppen Event ID (für group-Modus)
    pub nostr_group_event_id: Option<String>,
    /// Nostr Gruppen-Relay (für group-Modus)
    pub nostr_group_relay: Option<String>,
}

impl Config {
    /// Erstellt eine neue Config aus Umgebungsvariablen mit besserer Fehlerbehandlung
    pub fn from_env() -> Result<Self, ConfigError> {
        let telegram_bot_token = get_env_var("TELEGRAM_BOT_TOKEN")?;
        let telegram_group_id = get_env_var("TELEGRAM_GROUP_ID")?
            .parse::<i64>()
            .map_err(|_| ConfigError::InvalidValue {
                var: "TELEGRAM_GROUP_ID".to_string(),
                msg: "Muss eine gültige Zahl sein".to_string(),
            })?;
        let nostr_private_key = get_env_var("NOSTR_PRIVATE_KEY")?;
        
        // Encryption type mit Default auf nip17
        let encryption_type_str = env::var("ENCRYPTION_TYPE")
            .unwrap_or_else(|_| "nip17".to_string());
        let encryption_type = EncryptionType::from_str(&encryption_type_str)?;

        // Nostr Public Key nur für verschlüsselte Modi erforderlich
        let nostr_public_key = match encryption_type {
            EncryptionType::Nip04 | EncryptionType::Nip17 => {
                Some(get_env_var("NOSTR_PUBLIC_KEY")?)
            }
            EncryptionType::Public | EncryptionType::Group => {
                env::var("NOSTR_PUBLIC_KEY").ok()
            }
        };

        let nostr_relays: Vec<String> = get_env_var("NOSTR_RELAYS")?
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        // Gruppenoptionen (optional)
        let nostr_group_event_id = env::var("NOSTR_GROUP_EVENT_ID").ok();
        let nostr_group_relay = env::var("NOSTR_GROUP_RELAY").ok();

        // Validierung
        if nostr_relays.is_empty() {
            return Err(ConfigError::InvalidValue {
                var: "NOSTR_RELAYS".to_string(),
                msg: "Mindestens ein Relay muss angegeben werden".to_string(),
            });
        }

        // Gruppen-spezifische Validierung
        if encryption_type == EncryptionType::Group {
            if nostr_group_event_id.is_none() {
                return Err(ConfigError::InvalidValue {
                    var: "NOSTR_GROUP_EVENT_ID".to_string(),
                    msg: "Für ENCRYPTION_TYPE=group muss NOSTR_GROUP_EVENT_ID gesetzt sein".to_string(),
                });
            }
            if nostr_group_relay.is_none() {
                return Err(ConfigError::InvalidValue {
                    var: "NOSTR_GROUP_RELAY".to_string(),
                    msg: "Für ENCRYPTION_TYPE=group muss NOSTR_GROUP_RELAY gesetzt sein".to_string(),
                });
            }
        }

        Ok(Self {
            telegram_bot_token,
            telegram_group_id,
            nostr_private_key,
            nostr_public_key,
            nostr_relays,
            encryption_type,
            nostr_group_event_id,
            nostr_group_relay,
        })
    }

    /// Gibt den Gruppen-Event-ID zurück (nur für group-Modus)
    pub fn get_group_event_id(&self) -> Option<&str> {
        self.nostr_group_event_id.as_deref()
    }

    /// Gibt den Gruppen-Relay zurück (nur für group-Modus)
    pub fn get_group_relay(&self) -> Option<&str> {
        self.nostr_group_relay.as_deref()
    }


    /// Prüft ob der Modus Verschlüsselung benötigt
    pub fn needs_encryption(&self) -> bool {
        matches!(self.encryption_type, EncryptionType::Nip04 | EncryptionType::Nip17)
    }
}

fn get_env_var(var_name: &str) -> Result<String, ConfigError> {
    env::var(var_name).map_err(|_| ConfigError::MissingEnvVar(var_name.to_string()))
}