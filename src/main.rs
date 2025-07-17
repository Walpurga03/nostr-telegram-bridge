use teloxide::prelude::*;
use teloxide::types::Message;
use dotenv::dotenv;
use nostr_sdk::prelude::*;
use nostr_sdk::Kind;
use tokio::signal;
use std::sync::Arc;
use thiserror::Error;
use log::{info, warn, error, debug};

mod config;
use crate::config::{Config, ConfigError, EncryptionType};

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
async fn init_nostr_client(keys: &Keys, relays: &[String], config: &Config) -> Result<Client> {
    let client = Client::new(keys);
    
    // Standard-Relays hinzufügen
    for url in relays {
        match client.add_relay(url.clone()).await {
            Ok(_) => info!("Relay hinzugefügt: {}", url),
            Err(e) => warn!("Fehler beim Hinzufügen des Relays {}: {}", url, e),
        }
    }
    
    // Gruppen-Relay separat hinzufügen (falls vorhanden)
    if let Some(group_relay) = config.get_group_relay() {
        match client.add_relay(group_relay.to_string()).await {
            Ok(_) => info!("Gruppen-Relay hinzugefügt: {}", group_relay),
            Err(e) => warn!("Fehler beim Hinzufügen des Gruppen-Relays {}: {}", group_relay, e),
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
    recipient_pubkey: Option<&PublicKey>,
    text: &str,
    config: &Config,
) -> Result<EventId> {
    debug!("Sende {:?} Nachricht: {}", config.encryption_type, &text[..text.len().min(50)]);

    let event_builder = match config.encryption_type {
        EncryptionType::Nip04 => {
            info!("Sende NIP-04 verschlüsselte Nachricht...");
            let recipient = recipient_pubkey.ok_or_else(|| 
                BridgeError::Config(ConfigError::InvalidValue {
                    var: "NOSTR_PUBLIC_KEY".to_string(),
                    msg: "Empfänger-Pubkey für NIP-04 erforderlich".to_string(),
                })
            )?;
            EventBuilder::encrypted_direct_msg(keys, *recipient, text, None)
                .map_err(|e| BridgeError::EventBuild(e.to_string()))?
        },
        EncryptionType::Nip17 => {
            info!("Sende NIP-17 private Nachricht...");
            let recipient = recipient_pubkey.ok_or_else(|| 
                BridgeError::Config(ConfigError::InvalidValue {
                    var: "NOSTR_PUBLIC_KEY".to_string(),
                    msg: "Empfänger-Pubkey für NIP-17 erforderlich".to_string(),
                })
            )?;
            // Verwende NIP-17 wenn verfügbar, sonst fallback auf NIP-04
            EventBuilder::encrypted_direct_msg(keys, *recipient, text, None)
                .map_err(|e| BridgeError::EventBuild(e.to_string()))?
        },
        EncryptionType::Public => {
            info!("Sende öffentliche Nachricht...");
            let public_text = format!("📱 Telegram-Weiterleitung:\n{}", text);
            EventBuilder::text_note(public_text, Vec::new())
        },
        EncryptionType::Group => {
            info!("Sende Gruppen-Nachricht...");
            let group_event_id = EventId::from_hex(
                config.get_group_event_id().ok_or_else(|| 
                    BridgeError::Config(ConfigError::InvalidValue {
                        var: "NOSTR_GROUP_EVENT_ID".to_string(),
                        msg: "Gruppen-Event-ID fehlt".to_string(),
                    })
                )?
            ).map_err(|e| BridgeError::EventBuild(e.to_string()))?;
        
            
            // NIP-29 Gruppen-Nachricht (Kind 9) - KORRIGIERT
            EventBuilder::new(
                Kind::Custom(9),
                text,
                vec![
                    Tag::event(group_event_id), // KORRIGIERT: Tag::event statt Tag::Event
                    Tag::Generic(
                        TagKind::Custom("h".to_string().into()), // KORRIGIERT: .into() für Cow<str>
                        vec![hex::encode(group_event_id.as_bytes())],
                    ),
                ],
            )
        }
    };
    
    let event = event_builder.to_event(keys)
        .map_err(|e| BridgeError::EventBuild(e.to_string()))?;
    
    let event_id = client.send_event(event).await?;
    info!("Nachricht ({:?}) an Nostr gesendet! Event-ID: {}", config.encryption_type, event_id);
    Ok(event_id)
}

/// Behandelt eingehende Telegram-Nachrichten
async fn handle_telegram_message(
    message: Message,
    client: Arc<Client>,
    config: Arc<Config>,
    keys: Arc<Keys>,
    recipient_pubkey: Option<PublicKey>,
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
        let dt = chrono::DateTime::from_timestamp(message.date.timestamp(), 0)
            .unwrap_or_else(|| chrono::Utc::now());
        let time_str = dt.format("%Y-%m-%d %H:%M:%S").to_string();
        let time_short = dt.format("%H:%M").to_string();

        // Formatiere die Nachricht mit Metadaten
        let formatted_message = match config.encryption_type {
            EncryptionType::Public => {
                format!(
                    "Von: {} ({})\n\n{}",
                    sender_name,
                    time_short,
                    text
                )
            },
            EncryptionType::Group => {
                format!(
                    "📱 Telegram → Nostr Gruppe\n👤 Von: {} ({})\n\n{}",
                    sender_name,
                    time_short,
                    text
                )
            },
            _ => {
                format!(
                    "📱 Telegram-Nachricht\n👤 Von: {}\n📅 Zeit: {}\n\n{}",
                    sender_name,
                    time_str,
                    text
                )
            }
        };

        if let Err(e) = send_to_nostr(&client, &keys, recipient_pubkey.as_ref(), &formatted_message, &config).await {
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
    info!("Verschlüsselungstyp: {:?}", config.encryption_type);

    // Nostr-Keys und Client initialisieren
    let keys = Arc::new(
        Keys::parse(&config.nostr_private_key)
            .map_err(|e| BridgeError::KeyParsing(e.to_string()))?
    );
    
    // Empfänger-Pubkey nur für verschlüsselte Modi
    let recipient_pubkey = if config.needs_encryption() {
        let pubkey_str = config.nostr_public_key.as_ref()
            .ok_or_else(|| BridgeError::Config(ConfigError::InvalidValue {
                var: "NOSTR_PUBLIC_KEY".to_string(),
                msg: "Empfänger-Pubkey für verschlüsselte Modi erforderlich".to_string(),
            }))?;
        Some(PublicKey::from_bech32(pubkey_str)
            .map_err(|e| BridgeError::KeyParsing(e.to_string()))?)
    } else {
        None
    };
    
    let client = Arc::new(init_nostr_client(&keys, &config.nostr_relays, &config).await?);

    info!("🚀 Bridge läuft ({:?})", config.encryption_type);
    info!("📱 Telegram-Gruppe: {}", config.telegram_group_id);
    
    match config.encryption_type {
        EncryptionType::Public => {
            info!("🌐 Öffentliche Nachrichten aktiviert");
        },
        EncryptionType::Group => {
            info!("👥 Gruppen-Modus aktiviert");
            if let Some(group_id) = config.get_group_event_id() {
                info!("🔗 Gruppen-Event-ID: {}", group_id);
            }
            if let Some(group_relay) = config.get_group_relay() {
                info!("📡 Gruppen-Relay: {}", group_relay);
            }
        },
        _ => {
            if let Some(ref pubkey) = config.nostr_public_key {
                info!("🔒 Nostr-Empfänger: {}", pubkey);
            }
        }
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