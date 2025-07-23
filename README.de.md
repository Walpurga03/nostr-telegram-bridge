[🇬🇧 English](README.md)
# nostr-telegram-bridge

Eine Bridge-Anwendung, die Nachrichten aus einer Telegram-Gruppe als Nostr-Nachrichten weiterleitet. Unterstützt NIP-04, NIP-17, öffentliche Nachrichten und Nostr-Gruppen (NIP-29).

## Features

- 📱 Telegram-Gruppe → Nostr Weiterleitung
- 🔒 **NIP-17** (Private Messages) – Standard
- 🔐 **NIP-04** (Legacy Encryption) – Kompatibilität
- 🌐 **Öffentliche Nachrichten** – Keine Verschlüsselung
- 👥 **Nostr-Gruppen (NIP-29)** – Gruppen-Chat
- 🔄 Multi-Relay-Support
- 🛑 Graceful Shutdown
- ⚙️ Konfiguration über `.env`

## Voraussetzungen

- Rust 1.70+
- Telegram Bot Token
- Nostr-Schlüssel (Private Key)
- Nostr Public Key (nur für verschlüsselte Nachrichten)
- Telegram-Gruppen-ID
- **Für Gruppen-Modus**: Nostr-Gruppen-Event-ID und Gruppen-Relay

## Installation

```bash
git clone https://github.com/Walpurga03/nostr-telegram-bridge.git
cd nostr-telegram-bridge
cargo build --release
cp .env.example .env
```

## Schnellstart

1. `.env` konfigurieren (siehe unten)
2. Bridge starten: `cargo run`
3. Telegram-Nachricht senden → Erscheint in Nostr

## Konfiguration

Erstelle eine `.env`-Datei:

```env
# Telegram-Konfiguration
TELEGRAM_BOT_TOKEN=1234567890:ABCdXXXXXXXXXXXXXXXXXXXXXXx
TELEGRAM_GROUP_ID=-1001XXXXXXXXXXXXx

# Nostr-Konfiguration
NOSTR_PRIVATE_KEY=nsec1abcdef...
NOSTR_PUBLIC_KEY=npub1abcdef...  # Nur für nip04/nip17

# Relay-Konfiguration
NOSTR_RELAYS=wss://relay.damus.io,wss://nos.lol,wss://relay.snort.social

# Verschlüsselungstyp
ENCRYPTION_TYPE=nip17

# Gruppen-Konfiguration (nur für ENCRYPTION_TYPE=group)
NOSTR_GROUP_EVENT_ID=dde39dbaf95c637ea8XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
NOSTR_GROUP_RELAY=wss://groups.0xchat.com
```

## Verschlüsselungstypen

| Typ      | Beschreibung                           | Empfänger nötig | Spezielle Config     |
|----------|----------------------------------------|-----------------|----------------------|
| `nip17`  | Moderne private Nachrichten (Standard) | ✅              | ❌                   |
| `nip04`  | Legacy-Verschlüsselung (Kompatibilität)| ✅              | ❌                   |
| `public` | Öffentliche Nachrichten                | ❌              | ❌                   |
| `group`  | Nostr-Gruppen (NIP-29)                 | ❌              | ✅ Event-ID + Relay  |

## Setup-Schritte

### 1. Telegram Bot erstellen
1. Sende `/newbot` an [@BotFather](https://t.me/BotFather)
2. Folge den Anweisungen und kopiere den Bot-Token
3. Füge den Bot zu deiner Telegram-Gruppe hinzu
4. Gebe dem Bot **Leserechte** in der Gruppe

### 2. Telegram-Gruppen-ID ermitteln
```bash
# Token ersetzen und ausführen
# Nur die Gruppen-ID anzeigen
curl -s "https://api.telegram.org/bot<DEIN_BOT_TOKEN>/getUpdates" | jq '.result[].message.chat.id'

# Gruppen-ID und Name anzeigen
curl -s "https://api.telegram.org/bot<DEIN_BOT_TOKEN>/getUpdates" | jq '.result[].message.chat | {title, id}'
```

### 3. Nostr-Schlüssel generieren
- **Online**: [nostrtool.com](https://nostrtool.com) 
- **CLI**: `nostr-cli` Tool verwenden

### 4. Für Gruppen-Modus: Nostr-Gruppe einrichten
- **Option A**: Bestehende Gruppe verwenden (z.B. über [0xchat](https://0xchat.com))
- **Option B**: Neue Gruppe erstellen

#### Gruppen-Event-ID finden
```bash
# In 0xchat: Gruppen-Info → Event-ID kopieren
# Format: 64 Zeichen Hex-String
# Beispiel: dde39dbaf95c637ea8785583e4c1a64be0462f3609695592c433ee6697b19815
```

## Verwendung

```bash
# Entwicklung
RUST_LOG=info cargo run

# Produktion
./target/release/nostr-telegram-bridge

# Debug-Modus (ausführliche Logs)
RUST_LOG=debug cargo run
```

### Nachrichtenfluss
1. **Telegram**: Nachricht in konfigurierte Gruppe senden
2. **Bridge**: Empfängt und formatiert Nachricht
3. **Nostr**: Nachricht wird entsprechend `ENCRYPTION_TYPE` weitergeleitet

## Sicherheit

- ❌ **Niemals** Private Keys oder Bot Token in Git committen
- 🔒 `.env`-Datei absichern: `chmod 600 .env`
- 🛡️ **NIP-17 verwenden** für beste Sicherheit
- 🔑 **Separate Schlüssel** für Development/Production
- 👥 **Gruppen-Berechtigung** prüfen

## Fehlerbehandlung

### Häufige Probleme

**❌ Telegram-Gruppen-ID Format**
```bash
TELEGRAM_GROUP_ID=-1001234567890  # ✅ Korrekt (negativ!)
TELEGRAM_GROUP_ID=1234567890      # ❌ Falsch (positiv)
```

**❌ Nostr-Keys Format**
```bash
NOSTR_PRIVATE_KEY=nsec1...  # ✅ Korrekt (nsec1 Präfix)
NOSTR_PUBLIC_KEY=npub1...   # ✅ Korrekt (npub1 Präfix)
```

**❌ Gruppen-Berechtigung**
```bash
# Bot ist nicht in der Nostr-Gruppe berechtigt
# Lösung: Gruppen-Admin muss Bot-Berechtigung erteilen
```

**❌ Relay-Verbindung**
```bash
# Gruppen-Relay testen
curl -I wss://groups.0xchat.com
# Sollte "101 Switching Protocols" zurückgeben
```

## Vergleich der Verschlüsselungstypen

### 🔒 NIP-17 (Empfohlen)
- ✅ Moderne Kryptografie
- ✅ Bessere Metadaten-Verschleierung
- ✅ Schutz vor Timing-Angriffen
- ✅ Zukunftssicher
- ⚠️ Neuere Clients erforderlich

### 🔐 NIP-04 (Legacy)
- ✅ Maximale Client-Kompatibilität
- ✅ Bewährte Technologie
- ⚠️ Ältere Kryptografie
- ⚠️ Metadaten-Leaks möglich

### 🌐 Öffentlich
- ✅ Keine Verschlüsselung nötig
- ✅ Maximale Kompatibilität
- ✅ Einfache Einrichtung
- ⚠️ Jeder kann mitlesen

### 👥 Gruppen (NIP-29)
- ✅ Gruppen-Chat-Funktionalität
- ✅ Keine Empfänger-Konfiguration nötig
- ✅ Skalierbar für viele Nutzer
- ✅ Moderierbar durch Admins
- ⚠️ Gruppen-Setup erforderlich
- ⚠️ NIP-29 Client-Support nötig

## Lizenz

MIT – Siehe [LICENSE](LICENSE)

## Support

- 🐛 **Issues**: [GitHub Issues](https://github.com/Walpurga03/nostr-telegram-bridge/issues)
- 🐾 **Nostr**: `npub192jd2dxlqwfnemzz8hsk77z2rn4de3thelw6suvtvqsl79d0udysxzuswy`

---

## 🙏 Unterstützen & Danke sagen

Wenn dir das Projekt gefällt und du Danke sagen möchtest, unterstütze die Entwicklung gerne mit:

- ⚡ **Lightning**: `aldo.barazutti@walletofsatoshi.com`
- ⚡ **Nostr zap**: [npub192jd2dxlqwfnemzz8hsk77z2rn4de3thelw6suvtvqsl79d0udysxzuswy]

Danke für deine Unterstützung! 🚀

---

**💡 Tipp**: Für den Einstieg empfehlen wir den **NIP-17 Modus** für private Nachrichten oder **Gruppen-Modus** für Community-Chats.