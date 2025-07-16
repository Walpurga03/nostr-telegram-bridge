use teloxide::prelude::*;
use teloxide::types::Message;
use dotenv::dotenv;
use nostr_sdk::prelude::*;
use tokio::signal;
use std::sync::Arc;
use thiserror::Error;

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
}

type Result<T> = std::result::Result<T, BridgeError>;

/// Initialisiert den Nostr-Client und fügt alle Relays hinzu
async fn init_nostr_client(keys: &Keys, relays: &[String]) -> Result<Client> {
    let client = Client::new(keys);
    
    for url in relays {
        if let Err(e) = client.add_relay(url).await {
            eprintln!("Warnung: Fehler beim Hinzufügen des Relays {}: {}", url, e);
        } else {
            println!("Relay hinzugefügt: {}", url);
        }
    }
    
    client.connect().await;
    println!("Nostr-Client verbunden mit {} Relays", relays.len());
    Ok(client)
}

/// Sendet eine Nachricht an Nostr
async fn send_to_nostr(
    client: &Client,
    keys: &Keys,
    recipient_pubkey: &PublicKey,
    text: &str,
) -> Result<EventId> {
    let event_builder = EventBuilder::encrypted_direct_msg(keys, *recipient_pubkey, text, None)
        .map_err(|e| BridgeError::Nostr(e))?;
    
    let event = event_builder.to_event(keys)
        .map_err(|e| BridgeError::Nostr(e))?;
    
    let event_id = client.send_event(event).await?;
    println!("Nachricht an Nostr gesendet! Event-ID: {}", event_id);
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
    // Nur Nachrichten aus der gewünschten Gruppe weiterleiten
    if message.chat.id.0 != config.telegram_group_id {
        return Ok(());
    }

    if let Some(text) = message.text() {
        // Formatiere die Nachricht mit Metadaten
        let formatted_message = format!(
            "📱 Telegram-Nachricht\n👤 Von: {}\n📅 Zeit: {}\n\n{}",
            message.from()
                .map(|u| u.full_name())
                .unwrap_or_else(|| "Unbekannt".to_string()),
            message.date.format("%Y-%m-%d %H:%M:%S"),
            text
        );

        if let Err(e) = send_to_nostr(&client, &keys, &recipient_pubkey, &formatted_message).await {
            eprintln!("Fehler beim Senden an Nostr: {}", e);
        }
    }

    Ok(())
}

/// Hauptfunktion: Telegram-Nachrichten empfangen und an Nostr weiterleiten
#[tokio::main]
async fn main() -> Result<()> {
    // .env laden
    dotenv().ok();
    
    // Konfiguration laden
    let config = Arc::new(Config::from_env()?);
    println!("Konfiguration geladen");

    // Nostr-Keys und Client initialisieren
    let keys = Arc::new(
        Keys::parse(&config.nostr_private_key)
            .map_err(|e| BridgeError::KeyParsing(e.to_string()))?
    );
    
    let recipient_pubkey = PublicKey::from_bech32(&config.nostr_public_key)
        .map_err(|e| BridgeError::KeyParsing(e.to_string()))?;
    
    let client = Arc::new(init_nostr_client(&keys, &config.nostr_relays).await?);

    println!("🚀 Bridge läuft. Nachrichten aus der Telegram-Gruppe werden an Nostr weitergeleitet.");
    println!("📱 Telegram-Gruppe: {}", config.telegram_group_id);
    println!("🔗 Nostr-Empfänger: {}", config.nostr_public_key);

    let bot = Bot::new(&config.telegram_bot_token);

    // Graceful shutdown Handler
    let shutdown_signal = async {
        signal::ctrl_c().await.expect("Fehler beim Installieren des Shutdown-Handlers");
        println!("\n🛑 Shutdown-Signal erhalten, Bridge wird beendet...");
    };

    // Telegram-Handler: Jede Nachricht aus der Gruppe an Nostr weiterleiten
    let handler = teloxide::repl(bot, move |message: Message| {
        let client = client.clone();
        let config = config.clone();
        let keys = keys.clone();
        let recipient_pubkey = recipient_pubkey;
        
        async move {
            if let Err(e) = handle_telegram_message(message, client, config, keys, recipient_pubkey).await {
                eprintln!("Fehler beim Verarbeiten der Nachricht: {}", e);
            }
            Ok(())
        }
    });

    // Entweder auf Shutdown-Signal oder Handler-Completion warten
    tokio::select! {
        _ = shutdown_signal => {
            println!("Bridge beendet.");
            Ok(())
        }
        result = handler => {
            result.map_err(|e| BridgeError::KeyParsing(e.to_string()))
        }
    }
}