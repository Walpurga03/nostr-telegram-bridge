use teloxide::prelude::*;
use teloxide::types::Message;
use dotenv::dotenv;
use nostr_sdk::prelude::*;
use tokio::signal;
use std::sync::Arc;
use thiserror::Error;
use log::{info, warn, error, debug};

mod config;
use crate::config::{Config, ConfigError};

#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("Konfigurationsfehler: {0}")]
    Config(#[from] ConfigError),
    #[error("Nostr-Fehler: {0}")]
    Nostr(#[from] nostr_sdk::client::Error),
    #[error("Schlüssel-Parsing-Fehler: {0}")]
    KeyParsing(String),
    #[error("Event-Build-Fehler: {0}")]
    EventBuild(String),
}

type Result<T> = std::result::Result<T, BridgeError>;

/// Initialisiert den Nostr-Client und fügt alle Relays hinzu
async fn init_nostr_client(keys: &Keys, relays: &[String]) -> Result<Client> {
    let client = Client::new(keys);
    
    for url in relays {
        match client.add_relay(url.clone()).await {
            Ok(_) => info!("Relay hinzugefügt: {}", url),
            Err(e) => warn!("Fehler beim Hinzufügen des Relays {}: {}", url, e),
        }
    }
    
    client.connect().await;
    info!("Nostr-Client verbunden mit {} Relays", relays.len());
    Ok(client)
}

/// Sendet eine Nachricht an Nostr mit flexibler Verschlüsselung
async fn send_to_nostr(
    client: &Client,
    keys: &Keys,
    recipient_pubkey: &PublicKey,
    text: &str,
    encryption_type: &str,
) -> Result<EventId> {
    debug!("Sende {} Nachricht: {}", encryption_type, &text[..text.len().min(50)]);
    
    let event_builder = match encryption_type {
        "nip04" => {
            info!("Sende NIP-04 verschlüsselte Nachricht...");
            // NIP-04 Methode
            EventBuilder::encrypted_direct_msg(keys, *recipient_pubkey, text, None)
                .map_err(|e| BridgeError::EventBuild(e.to_string()))?
        },
        "nip17" => {
            info!("Sende NIP-17 private Nachricht...");
            // Fallback auf NIP-04 wenn NIP-17 nicht verfügbar ist
            EventBuilder::encrypted_direct_msg(keys, *recipient_pubkey, text, None)
                .map_err(|e| BridgeError::EventBuild(e.to_string()))?
        },
        "public" => {
            info!("Sende öffentliche Nachricht...");
            let public_text = format!("📱 Telegram-Weiterleitung:\n{}", text);
            EventBuilder::text_note(public_text, Vec::new())
        },
        _ => return Err(BridgeError::Config(ConfigError::InvalidValue {
            var: "ENCRYPTION_TYPE".to_string(),
            msg: "Unbekannter Verschlüsselungstyp".to_string(),
        })),
    };
    
    let event = event_builder.to_event(keys)
        .map_err(|e| BridgeError::EventBuild(e.to_string()))?;
    
    let event_id = client.send_event(event).await?;
    info!("Nachricht ({}) an Nostr gesendet! Event-ID: {}", encryption_type, event_id);
    Ok(event_id)
}

/// Behandelt eingehende Telegram-Nachrichten
async fn handle_telegram_message(
    message: Message,
    client: Arc<Client>,
    config: Arc<Config>,
    keys: Arc<Keys>,
    recipient_pubkey: PublicKey,
) -> Result<()> {
    debug!("Nachricht empfangen von Chat-ID: {}", message.chat.id.0);

    // Nur Nachrichten aus der gewünschten Gruppe weiterleiten
    if message.chat.id.0 != config.telegram_group_id {
        debug!("Nachricht ignoriert - falsche Gruppe");
        return Ok(());
    }

    if let Some(text) = message.text() {
        let sender_name = message.from()
            .map(|u| u.full_name())
            .unwrap_or_else(|| "Unbekannt".to_string());

        info!("Verarbeite Nachricht von: {}", sender_name);

        // Telegram-Datum (Unix-Timestamp) in lesbares Format umwandeln
        #[allow(deprecated)]
        let dt = chrono::NaiveDateTime::from_timestamp(message.date.timestamp(), 0);
        let time_str = dt.format("%Y-%m-%d %H:%M:%S").to_string();
        let time_short = dt.format("%H:%M").to_string();

        // Formatiere die Nachricht mit Metadaten
        let formatted_message = if config.encryption_type == "public" {
            format!(
                "Von: {} ({})\n\n{}",
                sender_name,
                time_short,
                text
            )
        } else {
            format!(
                "📱 Telegram-Nachricht\n👤 Von: {}\n📅 Zeit: {}\n\n{}",
                sender_name,
                time_str,
                text
            )
        };

        if let Err(e) = send_to_nostr(&client, &keys, &recipient_pubkey, &formatted_message, &config.encryption_type).await {
            error!("Fehler beim Senden an Nostr: {}", e);
        }
    }

    Ok(())
}

/// Hauptfunktion: Telegram-Nachrichten empfangen und an Nostr weiterleiten
#[tokio::main]
async fn main() -> Result<()> {
    // Logging initialisieren
    env_logger::init();
    info!("Bridge startet...");
    
    // .env laden
    dotenv().ok();
    
    // Konfiguration laden
    let config = Arc::new(Config::from_env()?);
    info!("Konfiguration geladen");
    info!("Verschlüsselungstyp: {}", config.encryption_type);

    // Nostr-Keys und Client initialisieren
    let keys = Arc::new(
        Keys::parse(&config.nostr_private_key)
            .map_err(|e| BridgeError::KeyParsing(e.to_string()))?
    );
    
    let recipient_pubkey = if config.encryption_type == "public" {
        // Bei öffentlichen Nachrichten wird der Empfänger nicht gebraucht
        keys.public_key()
    } else {
        PublicKey::from_bech32(&config.nostr_public_key)
            .map_err(|e| BridgeError::KeyParsing(e.to_string()))?
    };
    
    let client = Arc::new(init_nostr_client(&keys, &config.nostr_relays).await?);

    info!("🚀 Bridge läuft ({})", config.encryption_type.to_uppercase());
    info!("📱 Telegram-Gruppe: {}", config.telegram_group_id);
    
    if config.encryption_type != "public" {
        info!("🔒 Nostr-Empfänger: {}", config.nostr_public_key);
    } else {
        info!("🌐 Öffentliche Nachrichten aktiviert");
    }

    let bot = Bot::new(&config.telegram_bot_token);

    // Graceful shutdown Handler
    let shutdown_signal = async {
        signal::ctrl_c().await.expect("Fehler beim Installieren des Shutdown-Handlers");
        info!("🛑 Shutdown-Signal erhalten, Bridge wird beendet...");
    };

    // Telegram-Handler
    let handler = teloxide::repl(bot, move |message: Message| {
        let client = client.clone();
        let config = config.clone();
        let keys = keys.clone();
        let recipient_pubkey = recipient_pubkey;
        
        async move {
            if let Err(e) = handle_telegram_message(message, client, config, keys, recipient_pubkey).await {
                error!("Fehler beim Verarbeiten der Nachricht: {}", e);
            }
            Ok(())
        }
    });

    // Entweder auf Shutdown-Signal oder Handler-Completion warten
    tokio::select! {
        _ = shutdown_signal => {
            info!("Bridge beendet.");
            Ok(())
        }
        _ = handler => {
            // Handler ist beendet (z.B. bei Fehler in teloxide)
            Ok(())
        }
    }
}