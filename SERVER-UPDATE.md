# Server-Update Anleitung

## Skripte auf den Server holen

```bash
# 1. Auf dem Server einloggen
ssh alex@Macmini-1

# 2. Ins Projekt-Verzeichnis wechseln
cd ~/nostr-telegram-bridge

# 3. Änderungen von GitHub holen
git pull origin master

# 4. Skripte ausführbar machen (falls nötig)
chmod +x start-bridge.sh stop-bridge.sh

# 5. Bridge in tmux starten
./start-bridge.sh
```

## Workflow: Entwicklung → Server

### Auf dem Entwicklungs-Rechner:
```bash
# 1. Code ändern
vim src/main.rs

# 2. Testen
./start-bridge.sh
tmux attach -t bridge

# 3. Wenn alles funktioniert: Committen und pushen
git add .
git commit -m "Beschreibung der Änderung"
git push origin master
```

### Auf dem Server:
```bash
# 1. Änderungen holen
cd ~/nostr-telegram-bridge
git pull origin master

# 2. Neu kompilieren und starten
./start-bridge.sh

# 3. Logs prüfen
tmux attach -t bridge
# Detach mit: Strg+B, dann D
```

## Zurück zum Production-Service

Wenn Sie mit dem Testen fertig sind:

```bash
# Bridge stoppen und Service starten
./stop-bridge.sh --start-service

# Oder manuell:
./stop-bridge.sh
sudo systemctl start nostr-telegram-bridge.service
```

## Schnell-Referenz

```bash
# Bridge starten (stoppt automatisch Service)
./start-bridge.sh

# Bridge stoppen
./stop-bridge.sh

# Bridge stoppen + Service starten
./stop-bridge.sh --start-service

# Logs anschauen
tmux attach -t bridge

# Service-Status
sudo systemctl status nostr-telegram-bridge.service
```
