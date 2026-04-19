# 🚀 Deployment-Anleitung für Server (Git-Workflow)

Diese Anleitung zeigt dir den einfachsten Weg: **Lokal pushen → Auf Server pullen**

## 📋 Voraussetzungen

**Lokal:**
- Git konfiguriert
- GitHub-Repository vorhanden

**Auf dem Server:**
- Linux-Server mit SSH-Zugang
- Git installiert
- Rust und Cargo installiert

---

## 🎯 Schritt-für-Schritt Anleitung

### 1️⃣ Lokal: Änderungen committen und pushen

```bash
# Auf deinem lokalen Rechner
cd /home/tower/projekt/github/repo/nostr-telegram-bridge

# Status prüfen
git status

# Alle Änderungen hinzufügen
git add .

# Commit mit aussagekräftiger Nachricht
git commit -m "Migration zu DM-Modus: Bidirektionale Bridge mit Persistenz"

# Auf GitHub pushen
git push origin main
# (oder 'master', je nach Branch-Name)
```

### 2️⃣ Server: Erstmaliges Setup

**Nur beim ersten Mal nötig!**

```bash
# Auf Server einloggen
ssh dein-user@dein-server.de

# Rust installieren (falls nicht vorhanden)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Repository klonen
cd ~
git clone https://github.com/DEIN-USERNAME/nostr-telegram-bridge.git
cd nostr-telegram-bridge
```

### 3️⃣ Server: .env-Datei erstellen

```bash
# Im Projektverzeichnis
cd ~/nostr-telegram-bridge

# .env-Datei erstellen
nano .env
```

Füge deine **Server-Konfiguration** ein:

```env
# Telegram-Konfiguration
TELEGRAM_BOT_TOKEN=dein_bot_token_hier
TELEGRAM_GROUP_ID=-1001234567890

# Nostr-Konfiguration (Bridge-Account)
NOSTR_PRIVATE_KEY=nsec1...dein_bridge_account...
NOSTR_DM_RECIPIENT=npub1...dein_persoenlicher_account...

# Nostr-Relays
NOSTR_RELAYS=wss://relay.damus.io,wss://relay.nostr.band,wss://nos.lol

# Verschlüsselungstyp
ENCRYPTION_TYPE=nip04

# Datenbank-Pfad
DATABASE_PATH=./bridge.db

# Log-Level
RUST_LOG=info
```

**Wichtig:** 
- `NOSTR_PRIVATE_KEY` = nsec des **Bridge-Accounts** (sendet Nachrichten)
- `NOSTR_DM_RECIPIENT` = npub deines **persönlichen Accounts** (empfängt Nachrichten)

Speichern: `Ctrl+O`, `Enter`, `Ctrl+X`

```bash
# .env-Datei schützen
chmod 600 .env
```

### 4️⃣ Server: Kompilieren und starten

```bash
# Release-Build erstellen (dauert beim ersten Mal ~5-10 Minuten)
cargo build --release

# Bridge starten
./target/release/nostr-telegram-bridge
```

---

## 🔄 Updates einspielen (der einfache Weg!)

**Jedes Mal, wenn du lokal Änderungen gemacht hast:**

### Lokal: Pushen

```bash
cd /home/tower/projekt/github/repo/nostr-telegram-bridge

git add .
git commit -m "Beschreibung deiner Änderungen"
git push origin main
```

### Server: Pullen und neu kompilieren

```bash
# Auf Server einloggen
ssh dein-user@dein-server.de

# Ins Projektverzeichnis
cd ~/nostr-telegram-bridge

# Bridge stoppen (falls sie läuft)
# Wenn mit tmux: Ctrl+C
# Wenn als Service: sudo systemctl stop nostr-bridge

# Neueste Änderungen holen
git pull origin main

# Neu kompilieren
cargo build --release

# Bridge neu starten
./target/release/nostr-telegram-bridge
```

**Das war's!** Die `.env`-Datei bleibt unverändert, da sie in `.gitignore` steht.

---

## 🔧 Persistente Ausführung mit tmux

### tmux installieren (falls nicht vorhanden)

```bash
sudo apt update
sudo apt install tmux
```

### Bridge in tmux-Session starten

```bash
# Neue tmux-Session erstellen
tmux new -s bridge

# Bridge starten
cd ~/nostr-telegram-bridge
./target/release/nostr-telegram-bridge

# Von Session trennen (Bridge läuft weiter)
# Drücke: Ctrl+B, dann D
```

### Wieder zur Session verbinden

```bash
# Liste aller Sessions
tmux ls

# Zur bridge-Session verbinden
tmux attach -t bridge

# Session beenden
# In der Session: Ctrl+C (Bridge stoppen), dann 'exit'
```

---

## 🔧 Systemd Service einrichten (Autostart)

### Service-Datei erstellen

```bash
sudo nano /etc/systemd/system/nostr-bridge.service
```

Inhalt (ersetze `dein-username` mit deinem Benutzernamen):

```ini
[Unit]
Description=Nostr-Telegram Bridge
After=network.target

[Service]
Type=simple
User=dein-username
WorkingDirectory=/home/dein-username/nostr-telegram-bridge
ExecStart=/home/dein-username/nostr-telegram-bridge/target/release/nostr-telegram-bridge
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Umgebungsvariablen aus .env laden
EnvironmentFile=/home/dein-username/nostr-telegram-bridge/.env

[Install]
WantedBy=multi-user.target
```

### Service aktivieren

```bash
# Service neu laden
sudo systemctl daemon-reload

# Service aktivieren (Autostart)
sudo systemctl enable nostr-bridge

# Service starten
sudo systemctl start nostr-bridge

# Status prüfen
sudo systemctl status nostr-bridge

# Logs anzeigen
sudo journalctl -u nostr-bridge -f
```

### Service verwalten

```bash
# Service stoppen
sudo systemctl stop nostr-bridge

# Service neu starten
sudo systemctl restart nostr-bridge

# Logs anzeigen
sudo journalctl -u nostr-bridge -n 100
```

### Updates mit systemd

```bash
# Auf Server
cd ~/nostr-telegram-bridge

# Service stoppen
sudo systemctl stop nostr-bridge

# Updates holen
git pull origin main

# Neu kompilieren
cargo build --release

# Service starten
sudo systemctl start nostr-bridge

# Status prüfen
sudo systemctl status nostr-bridge
```

---

## 📊 Monitoring

### Logs überwachen

```bash
# Mit systemd
sudo journalctl -u nostr-bridge -f

# Mit tmux
tmux attach -t bridge
```

### Datenbank-Statistiken

```bash
cd ~/nostr-telegram-bridge

# Gesamtzahl Nachrichten
sqlite3 bridge.db "SELECT COUNT(*) FROM message_mappings;"

# Nachrichten pro Richtung
sqlite3 bridge.db "SELECT direction, COUNT(*) FROM message_mappings GROUP BY direction;"

# Letzte 10 Nachrichten
sqlite3 bridge.db "SELECT * FROM message_mappings ORDER BY timestamp DESC LIMIT 10;"
```

---

## 🐛 Troubleshooting

### Bridge startet nicht

```bash
# Logs prüfen
sudo journalctl -u nostr-bridge -n 50

# Manuell mit Debug-Logs starten
cd ~/nostr-telegram-bridge
RUST_LOG=debug ./target/release/nostr-telegram-bridge
```

### Git-Konflikte beim Pull

```bash
# Lokale Änderungen verwerfen (Vorsicht!)
git reset --hard HEAD
git pull origin main

# Oder: Lokale Änderungen sichern
git stash
git pull origin main
git stash pop
```

### Kompilierung schlägt fehl

```bash
# Rust aktualisieren
rustup update

# Dependencies neu laden
cargo clean
cargo build --release
```

### Verbindungsprobleme

```bash
# Telegram-API erreichbar?
curl -I https://api.telegram.org

# Nostr-Relays erreichbar?
curl -I https://relay.damus.io

# Firewall-Regeln prüfen
sudo ufw status
```

---

## 📝 Workflow-Zusammenfassung

```
┌─────────────────────────────────────────────────────────┐
│                    LOKALER RECHNER                      │
│                                                         │
│  1. Code ändern                                         │
│  2. git add .                                           │
│  3. git commit -m "Beschreibung"                        │
│  4. git push origin main                                │
│                                                         │
└─────────────────────────────────────────────────────────┘
                            │
                            │ GitHub
                            ▼
┌─────────────────────────────────────────────────────────┐
│                        SERVER                           │
│                                                         │
│  1. ssh dein-user@server                                │
│  2. cd ~/nostr-telegram-bridge                          │
│  3. sudo systemctl stop nostr-bridge (falls Service)    │
│  4. git pull origin main                                │
│  5. cargo build --release                               │
│  6. sudo systemctl start nostr-bridge                   │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## ✅ Checkliste für Ersteinrichtung

- [ ] Server-Zugang via SSH funktioniert
- [ ] Git auf Server installiert
- [ ] Rust und Cargo auf Server installiert
- [ ] Repository auf Server geklont
- [ ] `.env`-Datei erstellt und konfiguriert
- [ ] `.env`-Datei mit `chmod 600` geschützt
- [ ] Projekt erfolgreich kompiliert
- [ ] Bridge manuell getestet
- [ ] tmux oder systemd-Service eingerichtet
- [ ] Test-Nachricht Telegram → Nostr funktioniert
- [ ] Test-Nachricht Nostr → Telegram funktioniert

---

## 💡 Tipps

1. **Verwende aussagekräftige Commit-Messages**, damit du später weißt, was geändert wurde
2. **Teste lokal**, bevor du pushst
3. **Erstelle Backups** der `bridge.db` vor größeren Updates
4. **Überwache die Logs** nach Updates auf Fehler
5. **Dokumentiere deine Server-Konfiguration** (welche Relays, welche Accounts)

---

## 🆘 Hilfe

Bei Problemen siehe auch:
- [SETUP.de.md](SETUP.de.md) - Detaillierte Setup-Anleitung
- [ERKLAERUNG.md](ERKLAERUNG.md) - Erklärung der Nostr-Konzepte
- [README.de.md](README.de.md) - Allgemeine Dokumentation
