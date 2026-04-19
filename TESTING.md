# Live-Testing Anleitung

## Voraussetzungen

Die Bridge läuft bereits in tmux (gestartet mit `./start-bridge.sh`).

## Test 1: Telegram → Nostr (Hauptfunktion)

### Schritt 1: Logs überwachen
```bash
tmux attach -t bridge
```

### Schritt 2: Nachricht in Telegram senden
1. Öffnen Sie Telegram auf Ihrem Handy/Desktop
2. Gehen Sie zur konfigurierten Gruppe (ID: `-1001543272186`)
3. Senden Sie eine Test-Nachricht, z.B.:
   ```
   Test 123 - Bridge funktioniert!
   ```

### Schritt 3: Logs prüfen
In der tmux-Session sollten Sie sehen:
```
[INFO] Verarbeite Nachricht von: IhrName
[INFO] Nachricht von IhrName: Test 123 - Bridge funktioniert!
[INFO] Sende NIP-04 verschlüsselte Nachricht...
[INFO] Nachricht (Nip04) an Nostr gesendet! Event-ID: abc123...
```

### Schritt 4: Auf Nostr prüfen
1. Öffnen Sie einen Nostr-Client (z.B. Damus, Amethyst, Primal)
2. Loggen Sie sich mit dem Empfänger-Account ein (`npub1s75xtquwc3z2ua6strnfj4n70dq0eqfhnq66seta0cv30fgga8rs4ehh2u`)
3. Prüfen Sie Ihre DMs - die Nachricht sollte dort ankommen

## Test 2: Nostr → Telegram (Bidirektional)

### Schritt 1: DM von Nostr senden
1. Öffnen Sie Ihren Nostr-Client
2. Loggen Sie sich mit dem Empfänger-Account ein
3. Senden Sie eine DM an den Bridge-Bot (der Account mit dem `NOSTR_PRIVATE_KEY`)
4. Nachricht z.B.: `Hallo von Nostr!`

### Schritt 2: Logs prüfen
```
[INFO] Nostr-DM empfangen von npub1s75...
[INFO] Nachricht an Telegram gesendet
```

### Schritt 3: In Telegram prüfen
Die Nachricht sollte in der Telegram-Gruppe erscheinen mit:
```
📨 Nostr-DM
👤 Von: npub1s75...

Hallo von Nostr!
```

## Test 3: Loop-Schutz testen

### Schritt 1: Mehrere Nachrichten schnell senden
Senden Sie 3-5 Nachrichten schnell hintereinander in Telegram.

### Schritt 2: Logs prüfen
Jede Nachricht sollte nur **einmal** verarbeitet werden:
```
[INFO] Nachricht von IhrName: Test 1
[INFO] Nachricht (Nip04) an Nostr gesendet! Event-ID: abc...
[INFO] Nachricht von IhrName: Test 2
[INFO] Nachricht (Nip04) an Nostr gesendet! Event-ID: def...
```

**Keine** Duplikate oder Loop-Meldungen sollten erscheinen.

## Test 4: Datenbank-Statistiken

### Schritt 1: Bridge neu starten
```bash
./stop-bridge.sh
./start-bridge.sh
```

### Schritt 2: Statistiken beim Start prüfen
```
[INFO] 📈 Datenbank-Statistiken: X Nachrichten (Y T→N, Z N→T)
```

Die Zahlen sollten mit der Anzahl der gesendeten Nachrichten übereinstimmen.

## Test 5: Fehlerbehandlung

### Test 5a: Falsche Gruppe
1. Senden Sie eine Nachricht in eine **andere** Telegram-Gruppe
2. Logs sollten zeigen:
   ```
   [DEBUG] Nachricht ignoriert - falsche Gruppe
   ```

### Test 5b: Relay-Ausfall
Die Bridge sollte weiterlaufen, auch wenn ein Relay nicht erreichbar ist:
```
[WARN] Fehler beim Hinzufügen des Relays wss://...: ...
[INFO] Nostr-Client verbunden mit X Relays
```

## Test 6: Graceful Shutdown

### Schritt 1: In tmux-Session wechseln
```bash
tmux attach -t bridge
```

### Schritt 2: Strg+C drücken
Die Bridge sollte sauber beenden:
```
[INFO] 🛑 Shutdown-Signal erhalten, Bridge wird beendet...
[INFO] Bridge beendet.
```

### Schritt 3: Neu starten
```bash
./start-bridge.sh
```

## Debugging-Tipps

### Mehr Logs anzeigen
```bash
# Bridge stoppen
./stop-bridge.sh

# Mit Debug-Logs starten
tmux new-session -d -s bridge "cd $(pwd) && RUST_LOG=debug cargo run --release"
tmux attach -t bridge
```

### Logs durchsuchen
```bash
# Letzte 50 Zeilen
tmux capture-pane -t bridge -p | tail -50

# Nach Fehlern suchen
tmux capture-pane -t bridge -p | grep ERROR

# Nach bestimmter Nachricht suchen
tmux capture-pane -t bridge -p | grep "Test 123"
```

### Datenbank prüfen
```bash
sqlite3 bridge.db "SELECT * FROM message_mappings ORDER BY timestamp DESC LIMIT 10;"
```

### Prozesse prüfen
```bash
# Läuft die Bridge?
ps aux | grep nostr-telegram-bridge

# Läuft der Service?
sudo systemctl status nostr-telegram-bridge.service
```

## Häufige Probleme beim Testen

### Problem: Keine Logs erscheinen
**Lösung:** 
```bash
tmux attach -t bridge
# Warten Sie 2-3 Sekunden nach dem Senden einer Nachricht
```

### Problem: "TerminatedByOtherGetUpdates"
**Lösung:**
```bash
./stop-bridge.sh
sudo systemctl stop nostr-telegram-bridge.service
./start-bridge.sh
```

### Problem: Nachricht kommt nicht auf Nostr an
**Prüfen:**
1. Ist der Empfänger-Pubkey korrekt? (in `.env`)
2. Sind die Relays erreichbar?
3. Logs zeigen "Nachricht an Nostr gesendet"?

### Problem: Nachricht kommt nicht in Telegram an
**Prüfen:**
1. Ist die Gruppen-ID korrekt? (in `.env`)
2. Hat der Bot Admin-Rechte in der Gruppe?
3. Logs zeigen "Nachricht an Telegram gesendet"?

## Performance-Test

### Viele Nachrichten senden
```bash
# In Telegram: Senden Sie 20-30 Nachrichten schnell hintereinander
```

**Erwartetes Verhalten:**
- Alle Nachrichten werden verarbeitet
- Keine Duplikate
- Keine Fehler
- CPU-Last bleibt niedrig

### Ressourcen überwachen
```bash
# CPU und RAM
top -p $(pgrep -f nostr-telegram-bridge)

# Oder mit htop
htop -p $(pgrep -f nostr-telegram-bridge)
```

## Erfolgreicher Test

✅ Telegram → Nostr funktioniert  
✅ Nostr → Telegram funktioniert  
✅ Keine Duplikate (Loop-Schutz)  
✅ Datenbank speichert Mappings  
✅ Graceful Shutdown funktioniert  
✅ Fehlerbehandlung funktioniert  

**Dann ist die Bridge produktionsreif!**

## Zurück zum Production-Service

Wenn alle Tests erfolgreich sind:

```bash
# Bridge stoppen
./stop-bridge.sh

# Service starten
sudo systemctl start nostr-telegram-bridge.service

# Status prüfen
sudo systemctl status nostr-telegram-bridge.service

# Logs überwachen
sudo journalctl -u nostr-telegram-bridge.service -f
```
