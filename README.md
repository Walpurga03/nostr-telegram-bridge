# nostr-telegram-bridge

Eine Bridge-Anwendung, die Nachrichten aus einer Telegram-Gruppe als Nostr-Nachrichten weiterleitet. Unterstützt NIP-04, NIP-17 und öffentliche Nachrichten.

## Features

- Telegram-Gruppe → Nostr Weiterleitung
- **NIP-17** (Private Messages) - Standard
- **NIP-04** (Legacy Encryption) - Kompatibilität
- **Öffentliche Nachrichten** - Keine Verschlüsselung
- Multi-Relay-Support
- Graceful Shutdown
- Konfiguration über `.env`

## Voraussetzungen

- Rust 1.70+
- Telegram Bot Token
- Nostr-Schlüssel (Private Key)
- Nostr Public Key (nur für verschlüsselte Nachrichten)
- Telegram-Gruppen-ID

## Installation

```bash
git clone https://github.com/yourusername/nostr-telegram-bridge.git
cd nostr-telegram-bridge
cargo build --release
cp .env.example .env
# .env-Datei bearbeiten
```

## Konfiguration

Erstellen Sie eine `.env`-Datei:

```env
TELEGRAM_BOT_TOKEN=1234567890:ABCdefGHIjklMNOpqrsTUVwxyz
TELEGRAM_GROUP_ID=-1001234567890
NOSTR_PRIVATE_KEY=nsec1abcdef...
NOSTR_PUBLIC_KEY=npub1abcdef...
NOSTR_RELAYS=wss://relay.damus.io,wss://nos.lol,wss://relay.snort.social
ENCRYPTION_TYPE=nip17
```

### Verschlüsselungstypen

| Typ | Beschreibung | Empfänger nötig |
|-----|-------------|-----------------|
| `nip17` | **Moderne private Nachrichten** (Standard) | ✅ |
| `nip04` | Legacy-Verschlüsselung (Kompatibilität) | ✅ |
| `public` | Öffentliche Nachrichten | ❌ |

### Setup-Schritte

1. **Telegram Bot erstellen**: `/newbot` an [@BotFather](https://t.me/botfather)
2. **Bot zur Gruppe hinzufügen**: Leserechte geben
3. **Gruppen-ID ermitteln**: 
   ```bash
   curl -s "https://api.telegram.org/bot<TOKEN>/getUpdates" | jq '.result[].message.chat.id'
   ```
4. **Nostr-Schlüssel generieren**: z.B. über [nostrtool.com](https://nostrtool.com/)

## Verwendung

```bash
# Entwicklung
RUST_LOG=info cargo run

# Produktion
./target/release/nostr-telegram-bridge
```

### Nachrichtenformat

#### NIP-17 / NIP-04 (Verschlüsselt)
```
📱 Telegram-Nachricht
👤 Von: Max Mustermann
📅 Zeit: 2024-01-15 14:30:25

Ursprünglicher Nachrichtentext...
```

#### Öffentliche Nachrichten
```
📱 Telegram-Weiterleitung:
Von: Max Mustermann (14:30)

Ursprünglicher Nachrichtentext...
```

### Verschlüsselungstyp wechseln

```bash
# NIP-17 (empfohlen)
echo "ENCRYPTION_TYPE=nip17" >> .env

# NIP-04 (legacy)
echo "ENCRYPTION_TYPE=nip04" >> .env

# Öffentlich
echo "ENCRYPTION_TYPE=public" >> .env
```

## Systemd-Service

```ini
[Unit]
Description=Nostr Telegram Bridge
After=network.target

[Service]
Type=simple
User=your-user
WorkingDirectory=/path/to/nostr-telegram-bridge
ExecStart=/path/to/nostr-telegram-bridge/target/release/nostr-telegram-bridge
Restart=always
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

## Sicherheit

- **Niemals** Private Keys oder Bot Token in Git committen
- `.env`-Datei: `chmod 600 .env`
- **NIP-17 verwenden** für beste Sicherheit
- Separate Schlüssel für Development/Production

## Fehlerbehandlung

```bash
# Debug-Logs
RUST_LOG=debug cargo run

# Häufige Probleme:
# - Gruppen-ID muss negativ sein (-100...)
# - Nostr-Keys: nsec/npub Format prüfen
# - Bei "public": NOSTR_PUBLIC_KEY kann leer bleiben
# - Verschlüsselungstyp: nip04/nip17/public
```

## Vergleich der Verschlüsselungstypen

### NIP-17 (Empfohlen)
- ✅ Moderne Kryptografie
- ✅ Bessere Metadaten-Verschleierung
- ✅ Schutz vor Timing-Angriffen
- ✅ Zukunftssicher
- ⚠️ Neuere Clients erforderlich

### NIP-04 (Legacy)
- ✅ Maximale Client-Kompatibilität
- ✅ Bewährte Technologie
- ⚠️ Ältere Kryptografie
- ⚠️ Metadaten-Leaks möglich

### Öffentlich
- ✅ Keine Verschlüsselung nötig
- ✅ Maximale Kompatibilität
- ✅ Einfache Einrichtung
- ⚠️ Jeder kann mitlesen

## Beispiel-Konfigurationen

### Für maximale Sicherheit (NIP-17)
```env
ENCRYPTION_TYPE=nip17
NOSTR_PUBLIC_KEY=npub1empfaenger...
```

### Für Kompatibilität (NIP-04)
```env
ENCRYPTION_TYPE=nip04
NOSTR_PUBLIC_KEY=npub1empfaenger...
```

### Für öffentliche Gruppen
```env
ENCRYPTION_TYPE=public
# NOSTR_PUBLIC_KEY nicht erforderlich
```

## Lizenz

MIT - Siehe [LICENSE](LICENSE)

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/nostr-telegram-bridge/issues)
- **Nostr**: Kontakt über Nostr (siehe Cargo.toml)

---

**Empfehlung**: Verwenden Sie NIP-17 für neue Installationen. NIP-04 nur für Legacy-Kompatibilität.
