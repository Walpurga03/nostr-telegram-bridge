use teloxide::prelude::*;
use teloxide::types::Message;
use dotenv::dotenv;
use nostr_sdk::prelude::*;
use nostr_sdk::Kind;
use tokio::signal;
use std::sync::Arc;
use thiserror::Error;
use log::{info, warn, error, debug};
use chrono::{NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

mod config;
use crate::config::{Config, ConfigError, EncryptionType};

mod database;
use crate::database::{Database, MessageMapping, MessageDirection};

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
            // NIP-29 Gruppen-Modus (Legacy-Unterstützung)
            // Hinweis: Dieser Modus ist für Nostr-Gruppen gedacht, nicht für DM-Bridge
            // Für DM-Bridge verwenden Sie EncryptionType::Nip04 oder Nip17
            info!("Sende Gruppen-Nachricht (NIP-29)...");
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
    db: Arc<Database>,
) -> Result<()> {
    debug!("Nachricht empfangen von Chat-ID: {}", message.chat.id.0);

    // Nur Nachrichten aus der gewünschten Gruppe weiterleiten
    if message.chat.id.0 != config.telegram_group_id {
        debug!("Nachricht ignoriert - falsche Gruppe");
        return Ok(());
    }

    // Loop-Schutz: Prüfen ob Nachricht bereits verarbeitet wurde
    let telegram_msg_id = message.id.0 as i64;
    if db.telegram_message_exists(message.chat.id.0, telegram_msg_id)
        .unwrap_or(false) {
        debug!("Nachricht bereits verarbeitet (Loop-Schutz): {}", telegram_msg_id);
        return Ok(());
    }

    if let Some(text) = message.text() {
        let sender_name = message.from()
            .map(|u| u.full_name())
            .unwrap_or_else(|| "Unbekannt".to_string());

        info!("Verarbeite Nachricht von: {}", sender_name);

        // HIER Logging ergänzen:
        info!("Nachricht von {}: {}", sender_name, text);

        // Telegram-Datum (Unix-Timestamp) in lesbares Format umwandeln
        let tz: Tz = env::var("TIMEZONE")
            .unwrap_or_else(|_| "Europe/Berlin".to_string())
            .parse()
            .unwrap_or(chrono_tz::Europe::Berlin);

        #[allow(deprecated)]
        let dt = NaiveDateTime::from_timestamp(message.date.timestamp(), 0);
        let local_dt = tz.from_utc_datetime(&dt);
        let time_str = local_dt.format("%Y-%m-%d %H:%M:%S").to_string();
        let time_short = local_dt.format("%H:%M").to_string();

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

        match send_to_nostr(&client, &keys, recipient_pubkey.as_ref(), &formatted_message, &config).await {
            Ok(event_id) => {
                // Erfolgreich gesendet - in Datenbank speichern
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;

                let recipient_pubkey_str = recipient_pubkey
                    .map(|pk| pk.to_bech32().unwrap_or_else(|_| "unknown".to_string()))
                    .unwrap_or_else(|| "public".to_string());

                let mapping = MessageMapping {
                    id: None,
                    telegram_chat_id: message.chat.id.0,
                    telegram_message_id: telegram_msg_id,
                    nostr_event_id: event_id.to_hex(),
                    nostr_recipient_pubkey: recipient_pubkey_str,
                    direction: MessageDirection::TelegramToNostr,
                    timestamp,
                };

                // Prüfen ob Event-ID bereits existiert (kann passieren wenn wir unser eigenes Event empfangen)
                if !db.nostr_event_exists(&event_id.to_hex()).unwrap_or(false) {
                    if let Err(e) = db.save_mapping(&mapping) {
                        error!("Fehler beim Speichern des Mappings: {}", e);
                    } else {
                        debug!("Mapping gespeichert: Telegram {} -> Nostr {}", telegram_msg_id, event_id);
                    }
                } else {
                    debug!("Event-ID bereits in Datenbank, überspringe Speicherung");
                }
            }
            Err(e) => {
                error!("Fehler beim Senden an Nostr: {}", e);
            }
        }
    }

    Ok(())
}

/// Sendet eine Nachricht an Telegram
async fn send_to_telegram(
    bot: &Bot,
    chat_id: i64,
    text: &str,
) -> std::result::Result<teloxide::types::Message, teloxide::RequestError> {
    let msg = bot.send_message(ChatId(chat_id), text).await?;
    Ok(msg)
}

/// Hört auf Nostr-Events und leitet sie an Telegram weiter
async fn listen_nostr_events(
    client: Arc<Client>,
    keys: Arc<Keys>,
    config: Arc<Config>,
    bot: Bot,
    db: Arc<Database>,
    recipient_pubkey: Option<PublicKey>,
) -> Result<()> {
    info!("Starte Nostr-Event-Listener...");

    // Nur für DM-Modi (NIP-04/NIP-17)
    if !config.needs_encryption() {
        info!("Nostr-Listener nur für DM-Modi aktiv");
        return Ok(());
    }

    let recipient = match recipient_pubkey {
        Some(pk) => pk,
        None => {
            warn!("Kein Empfänger-Pubkey konfiguriert, Nostr-Listener wird nicht gestartet");
            return Ok(());
        }
    };

    // Filter für DMs (Kind 4 = NIP-04 encrypted DMs)
    // VEREINFACHTER FILTER: Nur nach Author, OHNE p-tag Filter
    let bridge_pubkey = keys.public_key();
    
    // Filter: ALLE DMs VON unserem User (egal an wen)
    // WICHTIG: Ohne .since() um auch ältere Events zu empfangen (für Tests)
    let filter = Filter::new()
        .kind(Kind::EncryptedDirectMessage)
        .author(recipient) // DMs VON npub1hht9...
        .limit(10); // Nur die letzten 10 Events zum Testen

    info!("Subscribing mit TEST-Filter:");
    info!("  - Kind: EncryptedDirectMessage (4)");
    info!("  - Author (sender): {}", recipient.to_bech32().unwrap_or_default());
    info!("  - Pubkey filter: DEAKTIVIERT (empfange alle DMs vom User)");
    info!("  - Since: DEAKTIVIERT (empfange auch ältere Events)");
    info!("  - Limit: 10 (zum Testen)");

    let subscription_id = client.subscribe(vec![filter.clone()], None).await;
    info!("Nostr-Subscription aktiv mit ID: {:?}", subscription_id);
    info!("Bridge-Bot Pubkey: {}", bridge_pubkey.to_bech32().unwrap_or_default());
    info!("Erwarteter Sender: {}", recipient.to_bech32().unwrap_or_default());

    // Event-Stream verarbeiten
    let mut notifications = client.notifications();
    info!("Warte auf Notifications vom Relay-Pool...");
    
    while let Ok(notification) = notifications.recv().await {
        info!(">>> Notification empfangen: {:?}", notification);
        
        if let RelayPoolNotification::Event { event, .. } = notification {
            info!("Event empfangen! Kind: {:?}, Author: {}", event.kind, event.pubkey.to_bech32().unwrap_or_default());
            
            // Loop-Schutz: Prüfen ob Event bereits verarbeitet wurde
            let event_id_hex = event.id.to_hex();
            if db.nostr_event_exists(&event_id_hex).unwrap_or(false) {
                debug!("Nostr-Event bereits verarbeitet (Loop-Schutz): {}", event_id_hex);
                continue;
            }

            // Nur Events vom konfigurierten Empfänger
            if event.pubkey != recipient {
                warn!("Event von anderem Pubkey ignoriert: {} (erwartet: {})",
                    event.pubkey.to_bech32().unwrap_or_default(),
                    recipient.to_bech32().unwrap_or_default());
                continue;
            }

            // DM entschlüsseln
            // WICHTIG: Wir müssen mit dem PUBLIC KEY des SENDERS entschlüsseln (nicht recipient)
            // Da der Sender = recipient ist, verwenden wir recipient als Gegenpartei
            let secret_key = keys.secret_key().expect("Failed to get secret key");
            
            // Der Sender ist 'recipient' (npub1hht9...), wir entschlüsseln mit dessen Public Key
            match nip04::decrypt(secret_key, &event.pubkey, &event.content) {
                Ok(decrypted_content) => {
                    info!("Nostr-DM empfangen von {}", event.pubkey.to_bech32().unwrap_or_default());
                    info!("Entschlüsselter Inhalt: {}", decrypted_content);
                    
                    // Formatiere Nachricht für Telegram
                    let formatted_message = format!(
                        "📨 Nostr-DM\n👤 Von: {}\n\n{}",
                        recipient.to_bech32().unwrap_or_else(|_| "unknown".to_string()),
                        decrypted_content
                    );

                    // An Telegram senden
                    match send_to_telegram(&bot, config.telegram_group_id, &formatted_message).await {
                        Ok(telegram_msg) => {
                            info!("Nachricht an Telegram gesendet");
                            
                            // In Datenbank speichern
                            let timestamp = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64;

                            let mapping = MessageMapping {
                                id: None,
                                telegram_chat_id: config.telegram_group_id,
                                telegram_message_id: telegram_msg.id.0 as i64,
                                nostr_event_id: event_id_hex.clone(),
                                nostr_recipient_pubkey: recipient.to_bech32().unwrap_or_else(|_| "unknown".to_string()),
                                direction: MessageDirection::NostrToTelegram,
                                timestamp,
                            };

                            if let Err(e) = db.save_mapping(&mapping) {
                                error!("Fehler beim Speichern des Mappings: {}", e);
                            } else {
                                debug!("Mapping gespeichert: Nostr {} -> Telegram {}", event_id_hex, telegram_msg.id.0);
                            }
                        }
                        Err(e) => {
                            error!("Fehler beim Senden an Telegram: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Fehler beim Entschlüsseln der Nostr-DM: {}", e);
                }
            }
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
        let pubkey_str = config.nostr_dm_recipient.as_ref()
            .ok_or_else(|| BridgeError::Config(ConfigError::InvalidValue {
                var: "NOSTR_DM_RECIPIENT".to_string(),
                msg: "Empfänger-Pubkey für verschlüsselte Modi erforderlich".to_string(),
            }))?;
        Some(PublicKey::from_bech32(pubkey_str)
            .map_err(|e| BridgeError::KeyParsing(e.to_string()))?)
    } else {
        None
    };
    
    let client = Arc::new(init_nostr_client(&keys, &config.nostr_relays, &config).await?);

    // Datenbank initialisieren
    let db = Arc::new(
        Database::new(&config.database_path)
            .map_err(|e| BridgeError::Config(ConfigError::InvalidValue {
                var: "DATABASE_PATH".to_string(),
                msg: format!("Fehler beim Öffnen der Datenbank: {}", e),
            }))?
    );
    info!("📊 Datenbank initialisiert: {}", config.database_path);

    // Statistiken anzeigen
    if let Ok((total, t_to_n, n_to_t)) = db.get_stats() {
        info!("📈 Datenbank-Statistiken: {} Nachrichten ({} T→N, {} N→T)", total, t_to_n, n_to_t);
    }

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
            if let Some(ref pubkey) = config.nostr_dm_recipient {
                info!("🔒 Nostr-DM-Empfänger: {}", pubkey);
            }
        }
    }

    let bot = Bot::new(&config.telegram_bot_token);

    // Graceful shutdown Handler
    let shutdown_signal = async {
        signal::ctrl_c().await.expect("Fehler beim Installieren des Shutdown-Handlers");
        info!("🛑 Shutdown-Signal erhalten, Bridge wird beendet...");
    };

    // Telegram-Handler (Task 1: Telegram → Nostr)
    let telegram_bot = bot.clone();
    let telegram_client = client.clone();
    let telegram_config = config.clone();
    let telegram_keys = keys.clone();
    let telegram_db = db.clone();
    let telegram_recipient = recipient_pubkey;
    
    let telegram_task = tokio::spawn(async move {
        teloxide::repl(telegram_bot, move |message: Message| {
            let client = telegram_client.clone();
            let config = telegram_config.clone();
            let keys = telegram_keys.clone();
            let db = telegram_db.clone();
            let recipient_pubkey = telegram_recipient;
            
            async move {
                if let Err(e) = handle_telegram_message(message, client, config, keys, recipient_pubkey, db).await {
                    error!("Fehler beim Verarbeiten der Telegram-Nachricht: {}", e);
                }
                Ok(())
            }
        }).await;
    });

    // Nostr-Listener (Task 2: Nostr → Telegram)
    let nostr_client = client.clone();
    let nostr_keys = keys.clone();
    let nostr_config = config.clone();
    let nostr_bot = bot.clone();
    let nostr_db = db.clone();
    let nostr_recipient = recipient_pubkey;
    
    let nostr_task = tokio::spawn(async move {
        if let Err(e) = listen_nostr_events(
            nostr_client,
            nostr_keys,
            nostr_config,
            nostr_bot,
            nostr_db,
            nostr_recipient,
        ).await {
            error!("Fehler im Nostr-Listener: {}", e);
        }
    });

    // Auf Shutdown-Signal oder Task-Completion warten
    tokio::select! {
        _ = shutdown_signal => {
            info!("Bridge beendet.");
            Ok(())
        }
        _ = telegram_task => {
            info!("Telegram-Task beendet");
            Ok(())
        }
        _ = nostr_task => {
            info!("Nostr-Task beendet");
            Ok(())
        }
    }
}