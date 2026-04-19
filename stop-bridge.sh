#!/bin/bash
# Nostr-Telegram-Bridge Stop-Skript

# Farben für Ausgabe
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Nostr-Telegram-Bridge Stopper ===${NC}"

# tmux-Session beenden
if tmux has-session -t bridge 2>/dev/null; then
    echo -e "${YELLOW}Beende tmux-Session 'bridge'...${NC}"
    tmux kill-session -t bridge
    echo -e "${GREEN}✅ tmux-Session beendet${NC}"
else
    echo -e "${YELLOW}⚠️  Keine tmux-Session 'bridge' gefunden${NC}"
fi

# Systemd-Service starten (falls gewünscht)
if [ "$1" == "--start-service" ]; then
    echo -e "${GREEN}Starte systemd-Service...${NC}"
    sudo systemctl start nostr-telegram-bridge.service
    sleep 1
    sudo systemctl status nostr-telegram-bridge.service --no-pager
fi
