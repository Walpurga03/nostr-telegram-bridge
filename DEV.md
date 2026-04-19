# Entwicklungs-Dokumentation

## Problem: Telegram Bot-Konflikt

Telegram erlaubt nur **eine aktive `getUpdates`-Verbindung** pro Bot-Token. Wenn der systemd-Service läuft und Sie versuchen, die Bridge in tmux zu starten, erhalten Sie den Fehler:

```
ERROR teloxide::error_handlers] An error from the update listener: Api(TerminatedByOtherGetUpdates)
```

## Lösung: Skripte verwenden

### 1. Bridge in tmux starten

```bash
./start-bridge.sh
```

Dieses Skript:
- Stoppt automatisch den systemd-Service (falls aktiv)
- Beendet alte tmux-Sessions (falls vorhanden)
- Startet die Bridge in einer neuen tmux-Session namens "bridge"
- Zeigt die letzten Logs an

### 2. Bridge stoppen

```bash
./stop-bridge.sh
```

Optional: Service wieder starten
```bash
./stop-bridge.sh --start-service
```

### 3. tmux-Befehle

```bash
# Session anzeigen (attach)
tmux attach -t bridge

# Session verlassen (detach) - OHNE die Bridge zu beenden
# Drücken Sie: Strg+B, dann D

# Logs in Echtzeit anzeigen (während attached)
# Die Logs werden automatisch angezeigt

# Session von außen überwachen
tmux capture-pane -t bridge -p | tail -20
```

## Manuelle Befehle

### Service-Status prüfen
```bash
sudo systemctl status nostr-telegram-bridge.service
```

### Service stoppen
```bash
sudo systemctl stop nostr-telegram-bridge.service
```

### Service starten
```bash
sudo systemctl start nostr-telegram-bridge.service
```

### Laufende Prozesse prüfen
```bash
ps aux | grep nostr-telegram-bridge
```

### Prozess manuell beenden
```bash
pkill -f nostr-telegram-bridge
```

## Entwicklungs-Workflow

### 1. Code ändern
Bearbeiten Sie die Dateien in `src/`

### 2. Bridge neu starten
```bash
./stop-bridge.sh
./start-bridge.sh
```

### 3. Logs überwachen
```bash
tmux attach -t bridge
```

### 4. Debugging mit mehr Logs
```bash
# In start-bridge.sh ändern Sie:
RUST_LOG=debug cargo run --release
# oder direkt:
tmux new-session -d -s bridge "cd $(pwd) && RUST_LOG=debug cargo run --release"
```

### 5. Zurück zum Production-Service
```bash
./stop-bridge.sh --start-service
```

## Build-Befehle

### Debug-Build (schneller kompilieren, langsamer laufen)
```bash
cargo build
./target/debug/nostr-telegram-bridge
```

### Release-Build (langsamer kompilieren, schneller laufen)
```bash
cargo build --release
./target/release/nostr-telegram-bridge
```

### Direkt ausführen (kompiliert automatisch)
```bash
cargo run                    # Debug
cargo run --release          # Release
```

## Logs

### Log-Level setzen
```bash
RUST_LOG=debug cargo run     # Sehr detailliert
RUST_LOG=info cargo run      # Normal (empfohlen)
RUST_LOG=warn cargo run      # Nur Warnungen
RUST_LOG=error cargo run     # Nur Fehler
```

### Logs filtern
```bash
# Nur Nostr-Logs
RUST_LOG=nostr_telegram_bridge=debug cargo run

# Mehrere Module
RUST_LOG=nostr_telegram_bridge=debug,teloxide=info cargo run
```

## Häufige Probleme

### Problem: "TerminatedByOtherGetUpdates"
**Ursache:** Eine andere Instanz läuft bereits  
**Lösung:** `./start-bridge.sh` verwenden (stoppt automatisch andere Instanzen)

### Problem: tmux-Session existiert bereits
**Ursache:** Alte Session nicht beendet  
**Lösung:** `tmux kill-session -t bridge` oder `./start-bridge.sh` verwenden

### Problem: Keine Berechtigung für systemctl
**Ursache:** sudo-Passwort erforderlich  
**Lösung:** Passwort eingeben oder Service manuell stoppen

### Problem: Bridge startet nicht
**Ursache:** Konfigurationsfehler in `.env`  
**Lösung:** 
1. `.env` mit `.env.example` vergleichen
2. Logs prüfen: `tmux attach -t bridge`
3. Manuell testen: `RUST_LOG=debug cargo run`

## Produktions-Deployment

Nach erfolgreichen Tests:

```bash
# 1. Release-Build erstellen
cargo build --release

# 2. Binary kopieren
sudo cp target/release/nostr-telegram-bridge /opt/nostr-telegram-bridge/bin/

# 3. Service neu starten
sudo systemctl restart nostr-telegram-bridge.service

# 4. Status prüfen
sudo systemctl status nostr-telegram-bridge.service

# 5. Logs überwachen
sudo journalctl -u nostr-telegram-bridge.service -f
```

## Nützliche Aliase

Fügen Sie zu `~/.bashrc` hinzu:

```bash
alias bridge-start='cd ~/nostr-telegram-bridge && ./start-bridge.sh'
alias bridge-stop='cd ~/nostr-telegram-bridge && ./stop-bridge.sh'
alias bridge-attach='tmux attach -t bridge'
alias bridge-logs='tmux capture-pane -t bridge -p | tail -50'
alias bridge-status='sudo systemctl status nostr-telegram-bridge.service'
```

Dann neu laden:
```bash
source ~/.bashrc
```
