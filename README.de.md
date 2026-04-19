# nostr-telegram-bridge

Eine Bridge-Anwendung für **wechselseitige Kommunikation** zwischen einer Telegram-Gruppe und einem Nostr-User über DMs. Unterstützt **NIP-17 Gift Wrap** (empfohlen) und NIP-04 verschlüsselte Direktnachrichten mit Persistenz und Loop-Schutz.

## ✨ Features

- 🔄 **Wechselseitige Kommunikation**: Telegram-Gruppe ↔ Nostr-User-DMs
- 🔒 **NIP-17 Gift Wrap**: Moderne verschlüsselte DMs mit maximaler Privatsphäre
- 👤 **Profilnamen-Anzeige**: Zeigt Nostr-Profilnamen statt nur npub
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

**Wichtige Features:**
- ✅ **Wechselseitige Kommunikation** (Telegram ↔ Nostr)
- ✅ **NIP-17 Gift Wrap Support** (moderne verschlüsselte DMs)
- ✅ **Persistentes Nachrichten-Mapping** (SQLite)
- ✅ **Loop-Schutz** und Deduplizierung
- ✅ **Profilnamen-Anzeige** (holt Namen aus Nostr-Profil)

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
4. DM von Nostr senden → Erscheint in Telegram-Gruppe mit Profilnamen

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

# Verschlüsselungstyp
ENCRYPTION_TYPE=nip17

# Datenbank-Pfad (optional, Standard: ./bridge.db)
DATABASE_PATH=./bridge.db
```

## 🔐 NIP-17 Gift Wrap Verschlüsselung

Die Bridge verwendet **NIP-17 Gift Wrap** für maximale Privatsphäre:

**Vorteile von NIP-17:**
- 🔒 **Maximale Privatsphäre**: Versteckt Metadaten (Sender, Empfänger, Zeitstempel)
- 🎁 **3-stufige Verschlüsselung**: Gift Wrap → Seal → Rumor
- 🔐 **nip44 Verschlüsselung**: Moderne, sichere Verschlüsselung
- 📱 **Moderne Clients**: Kompatibel mit 0xchat, Amethyst, und anderen aktuellen Nostr-Apps

**Wie funktioniert NIP-17?**
1. **Gift Wrap (Kind 1059)**: Äußere Verschlüsselung versteckt alle Metadaten
2. **Seal Event (Kind 13)**: Mittlere Schicht mit Sender-Informationen
3. **Rumor**: Eigentliche Nachricht (ungesigntes Event)

```
Nachricht → Rumor → Seal (nip44) → Gift Wrap (nip44) → Relay
```

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
3. Bridge sendet verschlüsselte NIP-17 Gift Wrap DM an Nostr-Empfänger
4. Mapping wird in Datenbank gespeichert

**Nostr → Telegram:**
1. Nostr-User sendet NIP-17 DM an Bridge
2. Bridge empfängt und entschlüsselt Nachricht (Gift Wrap → Seal → Rumor)
3. Bridge holt Profilnamen des Absenders (display_name oder name)
4. Bridge prüft auf Duplikate
5. Bridge sendet Nachricht in Telegram-Gruppe mit Format:
   ```
   📨 Nostr-DM
   👤 Von: Max Mustermann (npub1...)
   
   Nachrichteninhalt
   ```
6. Mapping wird in Datenbank gespeichert

## 🗄️ Datenbank

Die Bridge verwendet SQLite zum Speichern von Nachrichten-Mappings.

### Warum ist eine Datenbank notwendig?

**Problem ohne Datenbank:**
- 🔄 **Nachrichten-Schleifen**: Eine Nachricht von Telegram → Nostr würde wieder zurück zu Telegram → Nostr → ... (Endlosschleife!)
- 📨 **Doppelte Nachrichten**: Beim Neustart würden alte Nachrichten erneut verarbeitet
- 🔍 **Keine Nachverfolgung**: Unmöglich zu wissen, welche Telegram-Nachricht zu welchem Nostr-Event gehört

**Lösung mit Datenbank:**
Die Bridge speichert jede weitergeleitete Nachricht mit ihrer ID:

```sql
message_mapping:
- telegram_chat_id          # Telegram-Gruppen-ID
- telegram_message_id        # Eindeutige Telegram-Nachrichten-ID
- nostr_event_id            # Eindeutige Nostr-Event-ID
- nostr_recipient_pubkey    # Empfänger auf Nostr
- direction                 # telegram_to_nostr oder nostr_to_telegram
- timestamp                 # Zeitstempel der Weiterleitung
```

**Konkrete Vorteile:**
- ✅ **Loop-Schutz**: Bevor eine Nachricht weitergeleitet wird, prüft die Bridge: "Habe ich diese Nachricht schon verarbeitet?" → Wenn ja, wird sie ignoriert
- ✅ **Duplikat-Vermeidung**: Beim Neustart werden alte Nachrichten nicht erneut gesendet
- ✅ **Nachrichten-Tracking**: Du kannst sehen, welche Telegram-Nachricht zu welchem Nostr-Event gehört
- ✅ **Reply-Support** (geplant): Antworten auf Nachrichten können korrekt zugeordnet werden
- ✅ **Statistiken**: Anzahl der weitergeleiteten Nachrichten pro Richtung

**Beispiel-Szenario:**
1. User sendet "Hallo" in Telegram → Bridge leitet zu Nostr weiter → Speichert Mapping in DB
2. Nostr-Relay sendet Event zurück an Bridge (normale Relay-Funktion)
3. Bridge prüft DB: "Habe ich dieses Event schon verarbeitet?" → Ja! → Ignoriert es
4. ✅ Keine Schleife!

**Speicherort**: `./bridge.db` (konfigurierbar über `DATABASE_PATH`)

## 🔒 Sicherheit

- ❌ **Niemals** Private Keys oder Bot Token in Git committen
- 🔒 `.env`-Datei absichern: `chmod 600 .env`
- 🛡️ **NIP-17 verwenden** für beste Privatsphäre
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

**❌ Verschlüsselungsfehler**
```bash
# Stelle sicher, dass ENCRYPTION_TYPE auf nip17 gesetzt ist:
ENCRYPTION_TYPE=nip17  # ✅ Korrekt

# Dein Nostr-Client muss NIP-17 unterstützen (0xchat, Amethyst, etc.)
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
# 4. ENCRYPTION_TYPE passt zu deinem Client
# 5. Datenbank ist beschreibbar
```

## 🔄 Migration von vorheriger Version

Wenn du von der gruppenbasierten Version upgraden möchtest:

1. **`.env` sichern**
2. **`.env` aktualisieren**:
   - `NOSTR_PUBLIC_KEY` → `NOSTR_DM_RECIPIENT` umbenennen
   - `ENCRYPTION_TYPE=nip17` setzen
   - `DATABASE_PATH=./bridge.db` hinzufügen (optional)
3. **Gruppen-spezifische Einstellungen entfernen** (außer du brauchst Legacy-Gruppen-Support)
4. **Bridge neu starten**

## 🚧 Roadmap

- [x] **NIP-17-Unterstützung** (moderne verschlüsselte DMs) ✅
- [x] **Profilnamen-Anzeige** (display_name aus Nostr-Profil) ✅
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

## 🔧 Technische Details

### NIP-17 Gift Wrap Implementierung

Die Bridge implementiert vollständige NIP-17 Unterstützung mit 3-stufiger Entschlüsselung:

1. **Gift Wrap (Kind 1059)**: Äußere Verschlüsselung mit nip44
2. **Seal Event (Kind 13)**: Mittlere Schicht mit Sender-Informationen
3. **Rumor**: Eigentliche Nachricht (ungesigntes Event)

```rust
// Vereinfachter Ablauf:
Gift Wrap (nip44) → Seal Event → Seal (nip44) → Rumor → Content
```

### Profilnamen-Abruf

Die Bridge holt automatisch Profilnamen aus Nostr-Metadaten (Kind 0):
- Versucht zuerst `display_name`
- Falls nicht vorhanden, verwendet `name`
- Fallback: npub1... (wenn kein Profil gefunden)
- Timeout: 3 Sekunden

**💡 Tipp**: Diese Bridge ist für **Eins-zu-Eins-Kommunikation** zwischen einer Telegram-Gruppe und einem einzelnen Nostr-User über DMs konzipiert. Für Gruppen-zu-Gruppen-Kommunikation verwende den Legacy-Modus `ENCRYPTION_TYPE=group`.
