Hier ist die aktualisierte README.md mit der Gruppen-Funktionalität:

```markdown
# nostr-telegram-bridge

Eine Bridge-Anwendung, die Nachrichten aus einer Telegram-Gruppe als Nostr-Nachrichten weiterleitet. Unterstützt NIP-04, NIP-17, öffentliche Nachrichten und **Nostr-Gruppen (NIP-29)**.

## Features

- Telegram-Gruppe → Nostr Weiterleitung
- **NIP-17** (Private Messages) - Standard
- **NIP-04** (Legacy Encryption) - Kompatibilität
- **Öffentliche Nachrichten** - Keine Verschlüsselung
- **Nostr-Gruppen (NIP-29)** - Gruppen-Chat - **NEU!**
- Multi-Relay-Support
- Graceful Shutdown
- Konfiguration über `.env`

## Voraussetzungen

- Rust 1.70+
- Telegram Bot Token
- Nostr-Schlüssel (Private Key)
- Nostr Public Key (nur für verschlüsselte Nachrichten)
- Telegram-Gruppen-ID
- **Für Gruppen-Modus**: Nostr-Gruppen-Event-ID und Gruppen-Relay

## Installation

```bash
git clone https://github.com/yourusername/nostr-telegram-bridge.git
cd nostr-telegram-bridge
cargo build --release
cp .env.example .env
```

## Konfiguration

Erstellen Sie eine `.env`-Datei:

```env
# Telegram-Konfiguration
TELEGRAM_BOT_TOKEN=1234567890:ABCdefGHIjklMNOpqrsTUVwxyz
TELEGRAM_GROUP_ID=-1001234567890

# Nostr-Konfiguration
NOSTR_PRIVATE_KEY=nsec1abcdef...
NOSTR_PUBLIC_KEY=npub1abcdef...  # Nur für nip04/nip17

# Relay-Konfiguration
NOSTR_RELAYS=wss://relay.damus.io,wss://nos.lol,wss://relay.snort.social

# Verschlüsselungstyp
ENCRYPTION_TYPE=nip17

# Gruppen-Konfiguration (nur für ENCRYPTION_TYPE=group)
NOSTR_GROUP_EVENT_ID=dde39dbaf95c637ea8785583e4c1a64be0462f3609695592c433ee6697b19815
NOSTR_GROUP_RELAY=wss://groups.0xchat.com
```

## Verschlüsselungstypen

| Typ      | Beschreibung                           | Empfänger nötig | Spezielle Config     |
|----------|----------------------------------------|-----------------|----------------------|
| `nip17`  | Moderne private Nachrichten (Standard)| ✅              | ❌                   |
| `nip04`  | Legacy-Verschlüsselung (Kompatibilität)| ✅              | ❌                   |
| `public` | Öffentliche Nachrichten                | ❌              | ❌                   |
| `group`  | Nostr-Gruppen (NIP-29) - **NEU!**     | ❌              | ✅ Event-ID + Relay  |

## Setup-Schritte

### 1. Telegram Bot erstellen
```bash
# Telegram: /newbot an @BotFather
# Bot zur Gruppe hinzufügen: Leserechte geben
```

### 2. Gruppen-ID ermitteln
```bash
curl -s "https://api.telegram.org/bot<TOKEN>/getUpdates" | jq '.result[].message.chat.id'
```

### 3. Nostr-Schlüssel generieren
- Über [nostrtool.com](https://nostrtool.com) oder
- Mit `nostr-cli` Tool

### 4. Für Gruppen-Modus: Nostr-Gruppe setup
- **Option A**: Bestehende Gruppe verwenden (z.B. über 0xchat)
- **Option B**: Neue Gruppe erstellen (siehe Gruppen-Setup)

## Verwendung

```bash
# Entwicklung
RUST_LOG=info cargo run

# Produktion
./target/release/nostr-telegram-bridge

# Debug-Modus
RUST_LOG=debug cargo run
```

## Nachrichtenformate

### NIP-17 / NIP-04 (Verschlüsselt)
```
📱 Telegram-Nachricht
👤 Von: Max Mustermann
📅 Zeit: 2024-01-15 14:30:25

Ursprünglicher Nachrichtentext...
```

### Öffentliche Nachrichten
```
📱 Telegram-Weiterleitung:
Von: Max Mustermann (14:30)

Ursprünglicher Nachrichtentext...
```

### Nostr-Gruppen (NIP-29) - **NEU!**
```
📱 Telegram → Nostr Gruppe
👤 Von: Max Mustermann (14:30)

Ursprünglicher Nachrichtentext...
```

## Konfigurationsbeispiele

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

### Für Nostr-Gruppen (NIP-29) - **NEU!**
```env
ENCRYPTION_TYPE=group
NOSTR_GROUP_EVENT_ID=dde39dbaf95c637ea8785583e4c1a64be0462f3609695592c433ee6697b19815
NOSTR_GROUP_RELAY=wss://groups.0xchat.com
# NOSTR_PUBLIC_KEY nicht erforderlich
```

## Gruppen-Setup (NIP-29)

### Bestehende Gruppe verwenden
1. **0xchat öffnen**: [0xchat.com](https://0xchat.com)
2. **Gruppe beitreten**: Gruppen-Link verwenden
3. **Event-ID kopieren**: Aus Gruppen-Info
4. **Relay notieren**: Meist `wss://groups.0xchat.com`

### Neue Gruppe erstellen
```bash
# Mit 0xchat Client:
# 1. "Create Group" klicken
# 2. Name und Beschreibung eingeben
# 3. Event-ID aus URL kopieren
# 4. Relay-URL notieren
```

### Gruppen-Event-ID finden
```bash
# In 0xchat: Gruppen-Info → Event-ID
# Format: 64 Zeichen Hex-String
dde39dbaf95c637ea8785583e4c1a64be0462f3609695592c433ee6697b19815
```

## Verschlüsselungstyp wechseln

```bash
# NIP-17 (empfohlen)
echo "ENCRYPTION_TYPE=nip17" >> .env

# NIP-04 (legacy)
echo "ENCRYPTION_TYPE=nip04" >> .env

# Öffentlich
echo "ENCRYPTION_TYPE=public" >> .env

# Gruppen-Modus - NEU!
echo "ENCRYPTION_TYPE=group" >> .env
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

- ✅ Niemals Private Keys oder Bot Token in Git committen
- ✅ `.env`-Datei: `chmod 600 .env`
- ✅ NIP-17 verwenden für beste Sicherheit
- ✅ Separate Schlüssel für Development/Production
- ✅ Gruppen-Berechtigung prüfen

## Monitoring und Logs

```bash
# Debug-Logs
RUST_LOG=debug cargo run

# Wichtige Log-Nachrichten:
# ✅ "👥 Gruppen-Modus aktiviert"
# ✅ "📡 Gruppen-Relay: wss://groups.0xchat.com"
# ✅ "🔗 Gruppen-Event-ID: dde39..."
# ✅ "Nachricht (Group) an Nostr gesendet!"
```

## Fehlerbehandlung

### Häufige Probleme:

**Gruppen-ID muss negativ sein:**
```bash
TELEGRAM_GROUP_ID=-1001234567890  # ✅ Korrekt
TELEGRAM_GROUP_ID=1234567890      # ❌ Falsch
```

**Nostr-Keys Format prüfen:**
```bash
NOSTR_PRIVATE_KEY=nsec1...  # ✅ Korrekt
NOSTR_PUBLIC_KEY=npub1...   # ✅ Korrekt
```

**Gruppen-Berechtigung:**
```bash
# Prüfen Sie, ob Ihr Bot in der Nostr-Gruppe schreiben darf
# Gruppen-Admin muss Bot-Berechtigung erteilen
```

**Relay-Verbindung:**
```bash
# Gruppen-Relay testen
curl -I wss://groups.0xchat.com
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

### Gruppen (NIP-29) - **NEU!**
- ✅ Gruppen-Chat-Funktionalität
- ✅ Keine Empfänger-Konfiguration nötig
- ✅ Skalierbar für viele Nutzer
- ✅ Moderierbar durch Admins
- ⚠️ Gruppen-Setup erforderlich
- ⚠️ NIP-29 Client-Support nötig

## Lizenz

MIT - Siehe LICENSE

## Support

- **Issues**: GitHub Issues
- **Nostr**: Kontakt über Nostr (siehe Cargo.toml)
- **Telegram**: Community-Support

---

**Empfehlung**: 
- Verwenden Sie **NIP-17** für private Nachrichten
- Verwenden Sie **Gruppen-Modus** für Community-Chats
- NIP-04 nur für Legacy-Kompatibilität
```

Die wichtigsten Ergänzungen:

1. **Gruppen-Modus** in der Feature-Liste
2. **Erweiterte Konfigurationstabelle** mit Gruppen-Zeile
3. **Gruppen-Setup Sektion** mit detaillierten Anweisungen
4. **Neues Nachrichtenformat** für Gruppen
5. **Konfigurationsbeispiel** für Gruppen-Modus
6. **Erweiterte Fehlerbehandlung** für Gruppen-spezifische Probleme
7. **Vergleichstabelle** mit Gruppen-Vor-/Nachteilen

Die README ist jetzt vollständig und erklärt alle vier Modi: NIP-04, NIP-17, Public und Groups!