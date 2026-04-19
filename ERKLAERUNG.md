# 🔑 Verständnis: nsec, npub und die Bridge

## 📚 Grundlagen

### Was ist ein nsec (Private Key)?
- **nsec** = Ihr **geheimer Schlüssel** (wie ein Passwort)
- Damit können Sie Nachrichten **senden** und **signieren**
- **NIEMALS teilen!**

### Was ist ein npub (Public Key)?
- **npub** = Ihr **öffentlicher Schlüssel** (wie Ihre E-Mail-Adresse)
- Andere können damit Nachrichten **an Sie senden**
- Kann öffentlich geteilt werden

### Zusammenhang
```
nsec1yy0v... (PRIVAT)  →  npub1s75x... (ÖFFENTLICH)
     ↓                           ↓
  "Ich bin es"              "Schreib mir"
```

---

## 🌉 Wie funktioniert die Bridge?

### Die Bridge hat ZWEI Rollen:

#### 1. Bridge als "Sender" (NOSTR_PRIVATE_KEY)
```env
NOSTR_PRIVATE_KEY=nsec1yy0v...
```
- Die Bridge **sendet** Nachrichten mit diesem nsec
- Die Bridge **empfängt** DMs, die an den zugehörigen npub gesendet werden
- **Das ist der Bridge-Account**

#### 2. Bridge als "Vermittler" (NOSTR_DM_RECIPIENT)
```env
NOSTR_DM_RECIPIENT=npub1xyz...
```
- Die Bridge **leitet** Telegram-Nachrichten an diesen npub weiter
- Die Bridge **empfängt** DMs von diesem npub und leitet sie an Telegram weiter
- **Das ist der Empfänger-Account**

---

## 🎯 Ihre Frage beantwortet

> "Muss ich meinen nsec und npub eingeben und dann bekomme ich mit allen Clients, wo ich mit diesem verbunden bin, die Nachrichten?"

### Antwort: Es gibt ZWEI Szenarien

### Szenario A: Bridge sendet an SICH SELBST ❌ (Verwirrend!)

```env
NOSTR_PRIVATE_KEY=nsec1yy0v...        # Bridge-Account
NOSTR_DM_RECIPIENT=npub1s75x...       # Gleicher Account!
```

**Was passiert:**
- Bridge sendet DMs **an sich selbst**
- Bridge empfängt DMs **von sich selbst**

**Problem:**
- Sie brauchen **ZWEI verschiedene Nostr-Accounts** zum Testen:
  - Account 1: Bridge (nsec1yy0v...)
  - Account 2: Ihr persönlicher Account
- Sonst senden Sie DMs an sich selbst = verwirrend!

**Nicht empfohlen für den Start!**

---

### Szenario B: Bridge sendet an ANDEREN USER ✅ (Empfohlen!)

```env
NOSTR_PRIVATE_KEY=nsec1yy0v...        # Bridge-Account (nur für Bridge)
NOSTR_DM_RECIPIENT=npub1IHR_ECHTER... # Ihr persönlicher Account
```

**Was passiert:**
1. **Telegram → Nostr:**
   - Jemand schreibt in Telegram-Gruppe: "Hallo!"
   - Bridge sendet DM an `npub1IHR_ECHTER...`
   - **Sie empfangen die DM in IHREM Nostr-Client** (mit Ihrem persönlichen nsec eingeloggt)

2. **Nostr → Telegram:**
   - Sie senden DM **an die Bridge** (npub1s75x...)
   - Bridge empfängt und leitet an Telegram-Gruppe weiter
   - Alle in der Gruppe sehen Ihre Nachricht

**Vorteil:**
- ✅ Klare Trennung: Bridge-Account ≠ Ihr Account
- ✅ Sie nutzen Ihren normalen Nostr-Client
- ✅ Keine Verwirrung

---

## 🔧 Praktisches Beispiel

### Ihre aktuelle Situation

**Sie haben:**
- Bridge-nsec: `nsec1yy0v...`
- Bridge-npub: `npub1s75x...` (gehört zu nsec1yy0v...)

**Sie brauchen:**
- Ihren **persönlichen** Nostr-Account (mit eigenem nsec/npub)

### Setup-Schritte

#### Option 1: Sie haben bereits einen persönlichen Nostr-Account

```env
# Bridge-Account (nur für die Bridge)
NOSTR_PRIVATE_KEY=nsec1yy0v...

# Ihr persönlicher Account
NOSTR_DM_RECIPIENT=npub1IHR_PERSOENLICHER_NPUB...

ENCRYPTION_TYPE=nip04
```

**Dann:**
1. Starten Sie die Bridge
2. Öffnen Sie Ihren Nostr-Client (mit Ihrem persönlichen nsec eingeloggt)
3. Telegram-Nachrichten erscheinen als DMs in Ihrem Client
4. Senden Sie DMs an `npub1s75x...` (Bridge) → erscheinen in Telegram

#### Option 2: Sie haben noch keinen persönlichen Account

**Erstellen Sie einen neuen:**
```bash
# Auf nostrtool.com oder mit CLI
# Generieren Sie: nsec1NEU... / npub1NEU...
```

**Dann:**
```env
NOSTR_PRIVATE_KEY=nsec1yy0v...        # Bridge (bleibt)
NOSTR_DM_RECIPIENT=npub1NEU...        # Ihr neuer Account
ENCRYPTION_TYPE=nip04
```

**Und:**
1. Installieren Sie einen Nostr-Client (z.B. Amethyst, Damus, Gossip)
2. Loggen Sie sich mit `nsec1NEU...` ein
3. Fertig!

---

## 📱 Client-Frage beantwortet

> "Bekomme ich mit allen Clients, wo ich mit diesem verbunden bin, die Nachrichten?"

### Ja, ABER:

**Wenn Sie `NOSTR_DM_RECIPIENT=npub1IHR...` setzen:**
- Alle Clients, die mit **Ihrem persönlichen nsec** eingeloggt sind, empfangen die DMs
- Das können sein:
  - Amethyst auf Handy
  - Gossip auf Desktop
  - Damus auf iPhone
  - Alle gleichzeitig!

**Wichtig:**
- Die Clients müssen die **gleichen Relays** nutzen wie die Bridge
- Die Clients müssen **NIP-04 DMs** unterstützen (die meisten tun das)

---

## 🎯 Empfehlung für Sie

### Schritt 1: Entscheiden Sie

**Haben Sie bereits einen persönlichen Nostr-Account?**

**JA** → Nutzen Sie dessen npub als `NOSTR_DM_RECIPIENT`
**NEIN** → Erstellen Sie einen neuen Account

### Schritt 2: .env anpassen

```env
# Bridge-Account (bleibt wie es ist)
NOSTR_PRIVATE_KEY=nsec1yy0vmllu75ehkeaxg3auaersluj850ckeafdtrd4wwwap77g2q7sdfhp4c

# Ihr persönlicher Account (ÄNDERN!)
NOSTR_DM_RECIPIENT=npub1IHR_PERSOENLICHER_NPUB_HIER

# Relays (bleibt)
NOSTR_RELAYS=wss://nostr-relay.online,wss://relay.damus.io,wss://nostr.wine

# Modus ändern!
ENCRYPTION_TYPE=nip04

# Datenbank
DATABASE_PATH=./bridge.db
```

### Schritt 3: Testen

1. Bridge starten: `cargo run --release`
2. Nostr-Client öffnen (mit Ihrem persönlichen nsec)
3. Telegram-Nachricht senden → DM in Nostr-Client prüfen
4. DM an Bridge senden → Telegram-Gruppe prüfen

---

## ❓ Noch Fragen?

**F: Kann ich den Bridge-nsec auch als meinen persönlichen Account nutzen?**
A: Technisch ja, aber **nicht empfohlen**. Dann senden Sie DMs an sich selbst, was verwirrend ist.

**F: Muss ich einen neuen Account erstellen?**
A: Nur wenn Sie noch keinen persönlichen Nostr-Account haben. Wenn Sie bereits einen haben, nutzen Sie dessen npub.

**F: Sehen andere meine DMs?**
A: Nein! DMs sind verschlüsselt (NIP-04). Nur Sie und die Bridge können sie lesen.

**F: Kann ich mehrere Clients gleichzeitig nutzen?**
A: Ja! Alle Clients mit Ihrem nsec empfangen die DMs.

---

## 📝 Zusammenfassung

```
┌─────────────────────────────────────────────────────────────┐
│                    WIE ES FUNKTIONIERT                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Telegram-Gruppe                                            │
│       ↓                                                     │
│  Bridge (nsec1yy0v... / npub1s75x...)                      │
│       ↓                                                     │
│  Verschlüsselte DM                                          │
│       ↓                                                     │
│  Ihr Account (npub1IHR...)                                  │
│       ↓                                                     │
│  Ihre Nostr-Clients (Amethyst, Damus, Gossip, ...)         │
│                                                             │
│  UND ZURÜCK:                                                │
│                                                             │
│  Sie senden DM an Bridge (npub1s75x...)                    │
│       ↓                                                     │
│  Bridge empfängt                                            │
│       ↓                                                     │
│  Telegram-Gruppe                                            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Wichtig:** Bridge-Account ≠ Ihr persönlicher Account!
