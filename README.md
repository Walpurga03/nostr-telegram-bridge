# nostr-telegram-bridge

Eine minimalistische Bridge, die Nachrichten aus einer Telegram-Gruppe als Nostr-DM (NIP-04) an einen Empfänger weiterleitet.

## Features
- Empfängt Nachrichten aus einer Telegram-Gruppe
- Leitet sie als NIP-04-DM an einen Nostr-npub weiter
- Konfiguration über `.env`

## Einrichtung
1. **Repository klonen**
2. `.env.example` kopieren und als `.env` anpassen
3. Abhängigkeiten installieren:
   ```bash
   cargo build
   ```
4. Programm starten:
   ```bash
   cargo run
   ```

## Konfiguration (`.env`)
Siehe `.env.example` für alle nötigen Variablen:
- `TELEGRAM_BOT_TOKEN` – Token vom BotFather
- `TELEGRAM_GROUP_ID` – Telegram-Gruppen-ID (z.B. -100...)
- `NOSTR_PRIVATE_KEY` – nsec... (Absender)
- `NOSTR_PUBLIK_KEY` – npub... (Empfänger)
- `NOSTR_RELAYS` – Komma-separierte Liste von Relays

## Hinweise
- Der `target/`-Ordner und `.env` sind in `.gitignore` eingetragen.
- Der Author ist als Nostr-npub in der Cargo.toml hinterlegt.

## Lizenz
MIT
