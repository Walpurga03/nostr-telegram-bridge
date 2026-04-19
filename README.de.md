[🇬🇧 English](README.md)
# nostr-telegram-bridge

Eine **bidirektionale** Bridge-Anwendung, die Nachrichten zwischen einer Telegram-Gruppe und einem Nostr-User über DMs weiterleitet. Unterstützt NIP-04 verschlüsselte Direktnachrichten mit Persistenz und Loop-Schutz.

## ✨ Features

- 🔄 **Bidirektional**: Telegram-Gruppe ↔ Nostr-User-DMs
- 🔒 **NIP-04** verschlüsselte Direktnachrichten (Standard)
- 📊 **SQLite-Persistenz**: Nachrichten-Mapping und Deduplizierung
- 🛡️ **Loop-Schutz**: Verhindert Nachrichten-Schleifen
- 🔄 **Multi-Relay-Support**
- 🛑 **Graceful Shutdown**
- ⚙️ **Konfiguration über `.env`**
- 👥 **Legacy-Support**: NIP-29-Gruppen (optional)

## 🎯 Architektur

```
Telegram-Gruppe  ←→  Bridge  ←→  Nostr-User (DMs)
                       ↓
                   SQLite-DB
               (Nachrichten-Mapping)
```

**Wichtige Änderungen zur vorherigen Version:**
- ✅ Jetzt **bidirektional** (Telegram ↔ Nostr)
- ✅ **Persistentes Nachrichten-Mapping** (SQLite)
- ✅ **Loop-Schutz** und Deduplizierung
- ✅ Fokus auf **DM-basierte Kommunikation** (nicht Gruppen)
- ⚠️ NIP-17-Unterstützung geplant (aktuell NIP-04)

## 📋 Voraussetzungen

- Rust 1.70+
- Telegram Bot Token
- Nostr Private Key
- Nostr Empfänger Public Key (für DM-Modus)
- Telegram-Gruppen-ID

## 🚀 Installation

```bash
git clone https://github.com/Walpurga03/nostr-telegram-bridge.git
cd nostr-telegram-bridge
cargo build --release
cp .env.example .env
```

## ⚡ Schnellstart

1. `.env` konfigurieren (siehe unten)
2. Bridge starten: `cargo run`
3. Nachricht in Telegram-Gruppe senden → Erscheint als Nostr-DM
4. DM von Nostr senden → Erscheint in Telegram-Gruppe

## ⚙️ Konfiguration

Beispiel `.env`:

```env
# Telegram-Konfiguration
TELEGRAM_BOT_TOKEN=1234567890:ABCdXXXXXXXXXXXXXXXXXXXXXXx
TELEGRAM_GROUP_ID=-1001XXXXXXXXXXXXx

# Nostr-Konfiguration
NOSTR_PRIVATE_KEY=nsec1abcdef...
NOSTR_DM_RECIPIENT=npub1abcdef...  # Der Nostr-User für die Kommunikation

# Relay-Konfiguration
NOSTR_RELAYS=wss://relay.damus.io,wss://nos.lol,wss://relay.snort.social

# Verschlüsselungstyp (nip04 für DM-Bridge)
ENCRYPTION_TYPE=nip04

# Datenbank-Pfad (optional, Standard: ./bridge.db)
DATABASE_PATH=./bridge.db
```

## 🔐 Verschlüsselungstypen

| Typ      | Beschreibung                           | Anwendungsfall              | Status      |
|----------|----------------------------------------|-----------------------------|-------------|
| `nip04`  | Verschlüsselte DMs (NIP-04)            | **DM-Bridge** (empfohlen)   | ✅ Aktiv    |
| `nip17`  | Moderne private Nachrichten (NIP-17)   | Zukünftige DM-Bridge        | 🚧 Geplant  |
| `group`  | Nostr-Gruppen (NIP-29)                 | Legacy-Gruppen-Support      | ⚠️ Legacy   |
| `public` | Öffentliche Nachrichten                | Nur zum Testen              | ⚠️ Legacy   |

**Empfehlung**: Verwende `nip04` für die DM-Bridge. NIP-17-Unterstützung ist für zukünftige Versionen geplant.

## 📖 Setup-Schritte

### 1. Telegram Bot erstellen
1. Sende `/newbot` an [@BotFather](https://t.me/BotFather)
2. Folge den Anweisungen und kopiere den Bot-Token
3. Füge den Bot zu deiner Telegram-Gruppe hinzu
4. Gebe dem Bot **Leserechte** in der Gruppe

### 2. Telegram-Gruppen-ID ermitteln
```bash
# Token ersetzen und ausführen
curl -s "https://api.telegram.org/bot<DEIN_BOT_TOKEN>/getUpdates" | jq '.result[].message.chat.id'
```

### 3. Nostr-Schlüssel generieren
- **Online**: [nostrtool.com](https://nostrtool.com)
- **CLI**: `nostr-cli` Tool verwenden

### 4. Nostr-Empfänger Public Key ermitteln
- Der `npub1...` des Nostr-Users, mit dem du kommunizieren möchtest
- Dieser User erhält DMs von der Telegram-Gruppe
- Seine DMs werden in die Telegram-Gruppe weitergeleitet

## 🎮 Verwendung

```bash
# Entwicklung
RUST_LOG=info cargo run

# Produktion
./target/release/nostr-telegram-bridge

# Debug-Modus (ausführliche Logs)
RUST_LOG=debug cargo run
```

### Nachrichtenfluss

**Telegram → Nostr:**
1. User sendet Nachricht in Telegram-Gruppe
2. Bridge empfängt und prüft auf Duplikate
3. Bridge sendet verschlüsselte DM an Nostr-Empfänger
4. Mapping wird in Datenbank gespeichert

**Nostr → Telegram:**
1. Nostr-User sendet DM an Bridge
2. Bridge empfängt und entschlüsselt Nachricht
3. Bridge prüft auf Duplikate
4. Bridge sendet Nachricht in Telegram-Gruppe
5. Mapping wird in Datenbank gespeichert

## 🗄️ Datenbank

Die Bridge verwendet SQLite zum Speichern von Nachrichten-Mappings:

```sql
message_mapping:
- telegram_chat_id
- telegram_message_id
- nostr_event_id
- nostr_recipient_pubkey
- direction (telegram_to_nostr / nostr_to_telegram)
- timestamp
```

**Vorteile:**
- ✅ Loop-Schutz (verhindert doppelte Verarbeitung)
- ✅ Nachrichten-Tracking
- ✅ Reply-Support (geplant)
- ✅ Statistiken

**Speicherort**: `./bridge.db` (konfigurierbar über `DATABASE_PATH`)

## 🔒 Sicherheit

- ❌ **Niemals** Private Keys oder Bot Token in Git committen
- 🔒 `.env`-Datei absichern: `chmod 600 .env`
- 🛡️ **NIP-04 verwenden** für verschlüsselte DMs (NIP-17 kommt bald)
- 🔑 **Separate Schlüssel** für Development/Production
- 👥 **Gruppen-Berechtigung** prüfen
- 📊 Datenbank regelmäßig sichern

## 🐛 Fehlerbehandlung

### Häufige Probleme

**❌ Telegram-Gruppen-ID Format**
```bash
TELEGRAM_GROUP_ID=-1001234567890  # ✅ Korrekt (negativ!)
TELEGRAM_GROUP_ID=1234567890      # ❌ Falsch (positiv)
```

**❌ Nostr-Keys Format**
```bash
NOSTR_PRIVATE_KEY=nsec1...     # ✅ Korrekt (nsec1 Präfix)
NOSTR_DM_RECIPIENT=npub1...    # ✅ Korrekt (npub1 Präfix)
```

**❌ Datenbank gesperrt**
```bash
# Bei "database is locked" Fehlern:
# 1. Alle Bridge-Instanzen stoppen
# 2. bridge.db löschen
# 3. Bridge neu starten
```

**❌ Nachrichten werden nicht weitergeleitet**
```bash
# Logs prüfen:
RUST_LOG=debug cargo run

# Überprüfen:
# 1. Bot hat Leserechte in Telegram-Gruppe
# 2. Nostr-Relays sind erreichbar
# 3. Empfänger-Pubkey ist korrekt
# 4. Datenbank ist beschreibbar
```

## 🔄 Migration von vorheriger Version

Wenn du von der gruppenbasierten Version upgraden möchtest:

1. **`.env` sichern**
2. **`.env` aktualisieren**:
   - `NOSTR_PUBLIC_KEY` → `NOSTR_DM_RECIPIENT` umbenennen
   - `ENCRYPTION_TYPE=nip17` → `ENCRYPTION_TYPE=nip04` ändern
   - `DATABASE_PATH=./bridge.db` hinzufügen (optional)
3. **Gruppen-spezifische Einstellungen entfernen** (außer du brauchst Legacy-Gruppen-Support)
4. **Bridge neu starten**

## 🚧 Roadmap

- [ ] **NIP-17-Unterstützung** (moderne verschlüsselte DMs)
- [ ] **Reply-Support** (Telegram ↔ Nostr)
- [ ] **Medien-Support** (Bilder, Dateien)
- [ ] **Multi-User-Support** (mehrere Nostr-Empfänger)
- [ ] **Web-UI** für Konfiguration
- [ ] **Docker-Support**

## 📝 Lizenz

MIT – Siehe [LICENSE](LICENSE)

## 💬 Support

- 🐛 **Issues**: [GitHub Issues](https://github.com/Walpurga03/nostr-telegram-bridge/issues)
- 🐾 **Nostr**: `npub192jd2dxlqwfnemzz8hsk77z2rn4de3thelw6suvtvqsl79d0udysxzuswy`

---

## 🙏 Unterstützen & Danke sagen

Wenn dir das Projekt gefällt und du Danke sagen möchtest, unterstütze die Entwicklung gerne mit:

- ⚡ **Lightning**: `aldo.barazutti@walletofsatoshi.com`
- ⚡ **Nostr zap**: [npub192jd2dxlqwfnemzz8hsk77z2rn4de3thelw6suvtvqsl79d0udysxzuswy]

Danke für deine Unterstützung! 🚀

---

**💡 Tipp**: Diese Bridge ist für **Eins-zu-Eins-Kommunikation** zwischen einer Telegram-Gruppe und einem einzelnen Nostr-User über DMs konzipiert. Für Gruppen-zu-Gruppen-Kommunikation verwende den Legacy-Modus `ENCRYPTION_TYPE=group`.
