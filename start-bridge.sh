#!/bin/bash
# Nostr-Telegram-Bridge Startskript für tmux

# Farben für Ausgabe
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Nostr-Telegram-Bridge Starter ===${NC}"

# Prüfen ob systemd-Service läuft
if systemctl is-active --quiet nostr-telegram-bridge.service; then
    echo -e "${YELLOW}⚠️  Systemd-Service läuft noch!${NC}"
    echo -e "${YELLOW}Stoppe Service...${NC}"
    sudo systemctl stop nostr-telegram-bridge.service
    sleep 1
fi

# Prüfen ob bereits eine tmux-Session existiert
if tmux has-session -t bridge 2>/dev/null; then
    echo -e "${YELLOW}⚠️  tmux-Session 'bridge' existiert bereits${NC}"
    echo -e "${YELLOW}Beende alte Session...${NC}"
    tmux kill-session -t bridge
    sleep 1
fi

# Neue tmux-Session starten
echo -e "${GREEN}🚀 Starte Bridge in tmux-Session 'bridge'...${NC}"
tmux new-session -d -s bridge "cd $(pwd) && RUST_LOG=info cargo run --release"

# Kurz warten
sleep 2

# Status prüfen
if tmux has-session -t bridge 2>/dev/null; then
    echo -e "${GREEN}✅ Bridge läuft in tmux-Session 'bridge'${NC}"
    echo ""
    echo "Nützliche Befehle:"
    echo "  tmux attach -t bridge    # Session anzeigen"
    echo "  tmux detach              # Session verlassen (Strg+B, dann D)"
    echo "  tmux kill-session -t bridge  # Session beenden"
    echo ""
    echo -e "${YELLOW}Zeige letzte Logs:${NC}"
    tmux capture-pane -t bridge -p | tail -20
else
    echo -e "${RED}❌ Fehler beim Starten der Bridge${NC}"
    exit 1
fi
