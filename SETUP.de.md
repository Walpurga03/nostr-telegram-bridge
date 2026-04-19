# 🚀 Detaillierte Setup-Anleitung

Diese Anleitung führt Sie Schritt für Schritt durch die Einrichtung der Nostr-Telegram-Bridge.

## 📋 Voraussetzungen

- Linux/macOS/Windows mit WSL
- Rust installiert (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Ein Telegram-Account
- Ein Nostr-Account

---

## Schritt 1: Telegram Bot erstellen

### 1.1 Bot bei BotFather erstellen

1. Öffne Telegram und suche nach `@BotFather`
2. Starte einen Chat mit `/start`
3. Sende `/newbot`
4. Gib einen Namen für deinen Bot ein (z.B. "Meine Nostr Bridge")
5. Gib einen Username ein (muss auf `bot` enden, z.B. `meine_nostr_bridge_bot`)
6. **Kopiere den Bot Token** (sieht aus wie: `1234567890:ABCdefGHIjklMNOpqrsTUVwxyz`)

### 1.2 Bot zur Gruppe hinzufügen

1. Erstelle eine neue Telegram-Gruppe oder nutze eine bestehende
2. Füge deinen Bot zur Gruppe hinzu (über "Mitglied hinzufügen")
3. Gib dem Bot **Admin-Rechte** oder zumindest **Leserechte für Nachrichten**

### 1.3 Gruppen-ID ermitteln

**Option A: Mit curl (empfohlen)**
```bash
# Ersetze <DEIN_BOT_TOKEN> mit deinem Token
curl -s "https://api.telegram.org/bot<DEIN_BOT_TOKEN>/getUpdates" | jq '.result[].message.chat'
```

**Option B: Manuell**
1. Sende eine Nachricht in die Gruppe (z.B. "Test")
2. Öffne im Browser: `https://api.telegram.org/bot<DEIN_BOT_TOKEN>/getUpdates`
3. Suche nach `"chat":{"id":-1001234567890` (die negative Zahl ist deine Gruppen-ID)

**Wichtig**: Die Gruppen-ID ist **negativ** und beginnt meist mit `-100`

---

## Schritt 2: Nostr-Schlüssel generieren

### 2.1 Eigene Schlüssel für die Bridge

**Option A: Online (einfach)**
1. Gehe zu [nostrtool.com](https://nostrtool.com)
2. Klicke auf "Generate Keys"
3. **Kopiere den Private Key** (nsec1...) - **NIEMALS TEILEN!**
4. **Kopiere den Public Key** (npub1...) - kann geteilt werden

**Option B: CLI (sicher)**
```bash
# Mit nak (https://github.com/fiatjaf/nak)
nak key generate

# Oder mit nostr-tool
cargo install nostr-tool
nostr-tool generate
```

### 2.2 Empfänger-Pubkey ermitteln

Dies ist der **Nostr-User**, mit dem die Telegram-Gruppe kommunizieren soll.

**Wenn du selbst der Empfänger bist:**
- Nutze deinen eigenen Public Key (npub1...)

**Wenn jemand anderes der Empfänger ist:**
1. Frage nach dem Public Key (npub1...)
2. Oder finde ihn im Nostr-Client (siehe unten)

---

## Schritt 3: Nostr-Relays auswählen

### 3.1 Empfohlene Relays

**Standard-Relays (gut für den Start):**
```
wss://relay.damus.io
wss://nos.lol
wss://relay.snort.social
```

**Weitere beliebte Relays:**
```
wss://relay.nostr.band
wss://nostr.wine
wss://relay.primal.net
wss://nostr-pub.wellorder.net
```

### 3.2 Relay-Auswahl-Kriterien

- **Geschwindigkeit**: Teste mit `websocat wss://relay.damus.io` (muss schnell verbinden)
- **Verfügbarkeit**: Nutze mehrere Relays (3-5 empfohlen)
- **Geografische Nähe**: Wähle Relays in deiner Region
- **Bezahlte Relays**: Für bessere Performance (optional)

### 3.3 Relay-Status prüfen

**Online-Tools:**
- [nostr.watch](https://nostr.watch) - Relay-Status und Statistiken
- [relay.tools](https://relay.tools) - Relay-Tester

**CLI-Test:**
```bash
# Mit websocat
websocat wss://relay.damus.io
# Sollte sofort verbinden (Ctrl+C zum Beenden)
```

---

## Schritt 4: .env-Datei konfigurieren

### 4.1 Datei erstellen

```bash
cd nostr-telegram-bridge
cp .env.example .env
nano .env  # oder vim, code, etc.
```

### 4.2 Beispiel-Konfiguration

```env
# ===== TELEGRAM =====
TELEGRAM_BOT_TOKEN=1234567890:ABCdefGHIjklMNOpqrsTUVwxyz
TELEGRAM_GROUP_ID=-1001234567890

# ===== NOSTR =====
# Dein Bridge-Private-Key (GEHEIM HALTEN!)
NOSTR_PRIVATE_KEY=nsec1abcdefghijklmnopqrstuvwxyz1234567890abcdefghijklmnop

# Der Nostr-User, mit dem die Gruppe kommuniziert
NOSTR_DM_RECIPIENT=npub1xyz9876543210abcdefghijklmnopqrstuvwxyz1234567890abc

# Relays (komma-getrennt, KEINE Leerzeichen!)
NOSTR_RELAYS=wss://relay.damus.io,wss://nos.lol,wss://relay.snort.social

# ===== EINSTELLUNGEN =====
# Verschlüsselungstyp (nip04 für DM-Bridge)
ENCRYPTION_TYPE=nip04

# Datenbank-Pfad (optional)
DATABASE_PATH=./bridge.db

# ===== OPTIONAL: LOGGING =====
# Setze in der Shell vor dem Start:
# export RUST_LOG=info
```

### 4.3 Konfiguration validieren

**Checkliste:**
- [ ] `TELEGRAM_BOT_TOKEN` hat Format `NNNNNNNNNN:XXXXXXXXXXXXXXXXXXXXXXXXXXX`
- [ ] `TELEGRAM_GROUP_ID` ist **negativ** (beginnt mit `-`)
- [ ] `NOSTR_PRIVATE_KEY` beginnt mit `nsec1`
- [ ] `NOSTR_DM_RECIPIENT` beginnt mit `npub1`
- [ ] `NOSTR_RELAYS` hat **keine Leerzeichen** zwischen Relays
- [ ] `ENCRYPTION_TYPE=nip04` (nicht nip17!)

---

## Schritt 5: Nostr-Client auswählen

### 5.1 Empfohlene Clients für DMs

**Desktop:**
- **Gossip** (https://github.com/mikedilger/gossip) - Beste DM-Unterstützung
- **Nostrudel** (https://nostrudel.ninja) - Web-basiert
- **Coracle** (https://coracle.social) - Web-basiert

**Mobile:**
- **Amethyst** (Android) - Volle NIP-04-Unterstützung
- **Damus** (iOS) - Native DM-Unterstützung
- **Primal** (iOS/Android) - Einfach zu bedienen

**CLI:**
- **nak** (https://github.com/fiatjaf/nak) - Für Entwickler

### 5.2 Client-Konfiguration

**Wichtig**: Stelle sicher, dass dein Client die **gleichen Relays** nutzt wie die Bridge!

**In Gossip:**
1. Settings → Relays
2. Füge die Relays aus deiner `.env` hinzu
3. Aktiviere "Read" und "Write" für alle

**In Amethyst:**
1. Einstellungen → Relays
2. Füge Relays hinzu
3. Aktiviere für DMs

**In Damus:**
1. Settings → Relays
2. Add Relay
3. Füge alle Relays hinzu

### 5.3 DM-Test

1. Öffne deinen Nostr-Client
2. Suche nach dem **Bridge-Public-Key** (npub1... aus `NOSTR_PRIVATE_KEY`)
3. Sende eine Test-DM: "Hallo Bridge!"
4. Die Nachricht sollte in der Telegram-Gruppe erscheinen

---

## Schritt 6: Bridge starten

### 6.1 Erste Schritte

```bash
cd nostr-telegram-bridge

# Dependencies installieren
cargo build --release

# Logging aktivieren (optional)
export RUST_LOG=info

# Bridge starten
cargo run --release
```

### 6.2 Erwartete Ausgabe

```
[INFO] Bridge startet...
[INFO] Konfiguration geladen
[INFO] Verschlüsselungstyp: Nip04
[INFO] Datenbank initialisiert: ./bridge.db
[INFO] Relay hinzugefügt: wss://relay.damus.io
[INFO] Relay hinzugefügt: wss://nos.lol
[INFO] Relay hinzugefügt: wss://relay.snort.social
[INFO] Nostr-Client verbunden mit 3 Relays
[INFO] 🚀 Bridge läuft (Nip04)
[INFO] 📱 Telegram-Gruppe: -1001234567890
[INFO] 🔒 Nostr-DM-Empfänger: npub1xyz...
[INFO] Starte Nostr-Event-Listener...
[INFO] Nostr-Subscription aktiv für DMs von npub1xyz...
```

### 6.3 Fehlerbehandlung

**Fehler: "Umgebungsvariable 'TELEGRAM_BOT_TOKEN' fehlt"**
```bash
# Prüfe .env-Datei
cat .env | grep TELEGRAM_BOT_TOKEN
# Stelle sicher, dass keine Leerzeichen vor/nach = sind
```

**Fehler: "Fehler beim Hinzufügen des Relays"**
```bash
# Teste Relay manuell
websocat wss://relay.damus.io
# Wenn Fehler: Wähle anderen Relay
```

**Fehler: "Fehler beim Öffnen der Datenbank"**
```bash
# Prüfe Schreibrechte
touch bridge.db
rm bridge.db
# Oder ändere DATABASE_PATH in .env
```

---

## Schritt 7: Testen

### 7.1 Test: Telegram → Nostr

1. Sende eine Nachricht in die Telegram-Gruppe: "Test von Telegram"
2. Prüfe Bridge-Logs:
   ```
   [INFO] Verarbeite Nachricht von: Dein Name
   [INFO] Nachricht (Nip04) an Nostr gesendet! Event-ID: abc123...
   [INFO] Mapping gespeichert: Telegram 123 -> Nostr abc123...
   ```
3. Öffne deinen Nostr-Client
4. Prüfe DMs vom Bridge-Account
5. Du solltest die Nachricht sehen

### 7.2 Test: Nostr → Telegram

1. Öffne deinen Nostr-Client
2. Sende eine DM an den Bridge-Account: "Test von Nostr"
3. Prüfe Bridge-Logs:
   ```
   [INFO] Nostr-DM empfangen von npub1xyz...
   [INFO] Nachricht an Telegram gesendet
   [INFO] Mapping gespeichert: Nostr def456... -> Telegram 456
   ```
4. Prüfe die Telegram-Gruppe
5. Du solltest die Nachricht sehen

### 7.3 Test: Loop-Schutz

1. Sende eine Nachricht in Telegram
2. Die Nachricht erscheint in Nostr
3. Die Nachricht erscheint **NICHT** wieder in Telegram (Loop-Schutz aktiv!)
4. Prüfe Logs: `[DEBUG] Nachricht bereits verarbeitet (Loop-Schutz)`

---

## Schritt 8: Produktiv-Betrieb

### 8.1 Als Systemd-Service (Linux)

```bash
# Service-Datei erstellen
sudo nano /etc/systemd/system/nostr-bridge.service
```

```ini
[Unit]
Description=Nostr-Telegram Bridge
After=network.target

[Service]
Type=simple
User=dein-username
WorkingDirectory=/pfad/zu/nostr-telegram-bridge
Environment="RUST_LOG=info"
ExecStart=/pfad/zu/nostr-telegram-bridge/target/release/nostr-telegram-bridge
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

```bash
# Service aktivieren
sudo systemctl daemon-reload
sudo systemctl enable nostr-bridge
sudo systemctl start nostr-bridge

# Status prüfen
sudo systemctl status nostr-bridge

# Logs anzeigen
sudo journalctl -u nostr-bridge -f
```

### 8.2 Mit tmux/screen (einfach)

```bash
# Neue tmux-Session
tmux new -s bridge

# Bridge starten
cd nostr-telegram-bridge
RUST_LOG=info cargo run --release

# Detach: Ctrl+B, dann D
# Wieder anhängen: tmux attach -t bridge
```

### 8.3 Backup

```bash
# Datenbank sichern (täglich empfohlen)
cp bridge.db bridge.db.backup-$(date +%Y%m%d)

# Oder automatisch mit cron
crontab -e
# Füge hinzu:
0 2 * * * cp /pfad/zu/bridge.db /pfad/zu/backups/bridge.db.$(date +\%Y\%m\%d)
```

---

## 🔧 Erweiterte Konfiguration

### Mehrere Relays testen

```bash
# Relay-Performance testen
for relay in wss://relay.damus.io wss://nos.lol wss://relay.snort.social; do
  echo "Testing $relay..."
  timeout 2 websocat $relay && echo "✅ OK" || echo "❌ FAIL"
done
```

### Debug-Modus

```env
# In .env oder als Umgebungsvariable
RUST_LOG=debug
```

```bash
# Beim Start
RUST_LOG=debug cargo run
```

### Statistiken anzeigen

```bash
# SQLite-Datenbank abfragen
sqlite3 bridge.db "SELECT direction, COUNT(*) FROM message_mapping GROUP BY direction;"
```

---

## ❓ Häufige Fragen

**F: Kann ich mehrere Telegram-Gruppen verbinden?**
A: Nein, aktuell nur eine Gruppe pro Bridge-Instanz. Starte mehrere Instanzen mit verschiedenen `.env`-Dateien.

**F: Kann ich mehrere Nostr-User verbinden?**
A: Nein, aktuell nur ein Empfänger. Multi-User-Support ist geplant.

**F: Werden Medien (Bilder) unterstützt?**
A: Nein, aktuell nur Text. Medien-Support ist geplant.

**F: Ist NIP-17 verfügbar?**
A: Noch nicht vollständig. Aktuell wird NIP-04 verwendet. NIP-17-Upgrade ist vorbereitet.

**F: Wie sichere ich meine Private Keys?**
A: 
- Niemals in Git committen
- `.env` mit `chmod 600 .env` schützen
- Separate Keys für Test/Produktion
- Regelmäßige Backups

**F: Die Bridge empfängt keine Nostr-DMs**
A: Prüfe:
1. Gleiche Relays in Bridge und Client?
2. DM wirklich an Bridge-Pubkey gesendet?
3. Client unterstützt NIP-04?
4. Logs prüfen: `RUST_LOG=debug cargo run`

---

## 📞 Support

- **GitHub Issues**: https://github.com/Walpurga03/nostr-telegram-bridge/issues
- **Nostr**: npub192jd2dxlqwfnemzz8hsk77z2rn4de3thelw6suvtvqsl79d0udysxzuswy

---

**Viel Erfolg mit deiner Bridge! 🚀**
