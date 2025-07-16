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

/// Konfiguration für die Bridge, geladen aus Umgebungsvariablen
#[derive(Debug, Clone)]
pub struct Config {
    /// Telegram Bot Token (vom BotFather)
    pub telegram_bot_token: String,
    /// Telegram Gruppen-ID (z.B. -1001234567890)
    pub telegram_group_id: i64,
    /// Nostr Private Key (nsec...)
    pub nostr_private_key: String,
    /// Nostr Empfänger-Pubkey (npub...)
    pub nostr_public_key: String,
    /// Liste von Nostr-Relays
    pub nostr_relays: Vec<String>,
    /// Verschlüsselungstyp: "nip04", "nip17", "public"
    pub encryption_type: String,
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
        let nostr_public_key = get_env_var("NOSTR_PUBLIC_KEY")?;
        let nostr_relays: Vec<String> = get_env_var("NOSTR_RELAYS")?
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        // Encryption type mit Default auf nip17
        let encryption_type = env::var("ENCRYPTION_TYPE")
            .unwrap_or_else(|_| "nip17".to_string());

        // Validierung
        if nostr_relays.is_empty() {
            return Err(ConfigError::InvalidValue {
                var: "NOSTR_RELAYS".to_string(),
                msg: "Mindestens ein Relay muss angegeben werden".to_string(),
            });
        }

        // Verschlüsselungstyp validieren
        match encryption_type.as_str() {
            "nip04" | "nip17" | "public" => {},
            _ => return Err(ConfigError::InvalidValue {
                var: "ENCRYPTION_TYPE".to_string(),
                msg: "Muss 'nip04', 'nip17' oder 'public' sein".to_string(),
            }),
        }

        Ok(Self {
            telegram_bot_token,
            telegram_group_id,
            nostr_private_key,
            nostr_public_key,
            nostr_relays,
            encryption_type,
        })
    }
}

fn get_env_var(var_name: &str) -> Result<String, ConfigError> {
    env::var(var_name).map_err(|_| ConfigError::MissingEnvVar(var_name.to_string()))
}