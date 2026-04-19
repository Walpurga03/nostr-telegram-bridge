[🇩🇪 Deutsch](README.de.md)
# nostr-telegram-bridge

A **bidirectional** bridge application that forwards messages between a Telegram group and a Nostr user via DMs. Supports NIP-04 encrypted direct messages with persistence and loop protection.

## ✨ Features

- 🔄 **Bidirectional**: Telegram group ↔ Nostr user DMs
- 🔒 **NIP-04** encrypted direct messages (default)
- 📊 **SQLite persistence**: Message mapping and deduplication
- 🛡️ **Loop protection**: Prevents message loops
- 🔄 **Multi-relay support**
- 🛑 **Graceful shutdown**
- ⚙️ **Configuration via `.env`**
- 👥 **Legacy support**: NIP-29 groups (optional)

## 🎯 Architecture

```
Telegram Group  ←→  Bridge  ←→  Nostr User (DMs)
                      ↓
                  SQLite DB
              (message mapping)
```

**Key changes from previous version:**
- ✅ Now **bidirectional** (Telegram ↔ Nostr)
- ✅ **Persistent message mapping** (SQLite)
- ✅ **Loop protection** and deduplication
- ✅ Focus on **DM-based communication** (not groups)
- ⚠️ NIP-17 support planned (currently uses NIP-04)

## 📋 Requirements

- Rust 1.70+
- Telegram bot token
- Nostr private key
- Nostr recipient public key (for DM mode)
- Telegram group ID

## 🚀 Installation

```bash
git clone https://github.com/Walpurga03/nostr-telegram-bridge.git
cd nostr-telegram-bridge
cargo build --release
cp .env.example .env
```

## ⚡ Quickstart

1. Configure `.env` (see below)
2. Start the bridge: `cargo run`
3. Send a message in your Telegram group → It appears as Nostr DM
4. Send a DM from Nostr → It appears in the Telegram group

## ⚙️ Configuration

Example `.env`:

```env
# Telegram configuration
TELEGRAM_BOT_TOKEN=1234567890:ABCdXXXXXXXXXXXXXXXXXXXXXXx
TELEGRAM_GROUP_ID=-1001XXXXXXXXXXXXx

# Nostr configuration
NOSTR_PRIVATE_KEY=nsec1abcdef...
NOSTR_DM_RECIPIENT=npub1abcdef...  # The Nostr user to communicate with

# Relay configuration
NOSTR_RELAYS=wss://relay.damus.io,wss://nos.lol,wss://relay.snort.social

# Encryption type (nip04 for DM bridge)
ENCRYPTION_TYPE=nip04

# Database path (optional, default: ./bridge.db)
DATABASE_PATH=./bridge.db
```

## 🔐 Encryption types

| Type     | Description                          | Use case                    | Status      |
|----------|--------------------------------------|-----------------------------|-------------|
| `nip04`  | Encrypted DMs (NIP-04)               | **DM Bridge** (recommended) | ✅ Active   |
| `nip17`  | Modern private messages (NIP-17)     | Future DM Bridge            | 🚧 Planned  |
| `group`  | Nostr groups (NIP-29)                | Legacy group support        | ⚠️ Legacy   |
| `public` | Public messages                      | Testing only                | ⚠️ Legacy   |

**Recommendation**: Use `nip04` for the DM bridge. NIP-17 support is planned for future releases.

## 📖 Setup steps

### 1. Create a Telegram bot
1. Send `/newbot` to [@BotFather](https://t.me/BotFather)
2. Follow the instructions and copy the bot token
3. Add the bot to your Telegram group
4. Give the bot **read permissions** in the group

### 2. Get the Telegram group ID
```bash
# Replace <YOUR_BOT_TOKEN> and run
curl -s "https://api.telegram.org/bot<YOUR_BOT_TOKEN>/getUpdates" | jq '.result[].message.chat.id'
```

### 3. Generate Nostr keys
- **Online**: [nostrtool.com](https://nostrtool.com)
- **CLI**: Use the `nostr-cli` tool

### 4. Get the Nostr recipient public key
- The `npub1...` of the Nostr user you want to communicate with
- This user will receive DMs from the Telegram group
- Their DMs will be forwarded to the Telegram group

## 🎮 Usage

```bash
# Development
RUST_LOG=info cargo run

# Production
./target/release/nostr-telegram-bridge

# Debug mode (verbose logs)
RUST_LOG=debug cargo run
```

### Message flow

**Telegram → Nostr:**
1. User sends message in Telegram group
2. Bridge receives and checks for duplicates
3. Bridge sends encrypted DM to Nostr recipient
4. Mapping is stored in database

**Nostr → Telegram:**
1. Nostr user sends DM to bridge
2. Bridge receives and decrypts message
3. Bridge checks for duplicates
4. Bridge sends message to Telegram group
5. Mapping is stored in database

## 🗄️ Database

The bridge uses SQLite to store message mappings:

```sql
message_mapping:
- telegram_chat_id
- telegram_message_id
- nostr_event_id
- nostr_recipient_pubkey
- direction (telegram_to_nostr / nostr_to_telegram)
- timestamp
```

**Benefits:**
- ✅ Loop protection (prevents duplicate processing)
- ✅ Message tracking
- ✅ Reply support (planned)
- ✅ Statistics

**Location**: `./bridge.db` (configurable via `DATABASE_PATH`)

## 🔒 Security

- ❌ **Never** commit private keys or bot tokens to git
- 🔒 Secure your `.env` file: `chmod 600 .env`
- 🛡️ Use **NIP-04** for encrypted DMs (NIP-17 coming soon)
- 🔑 Use separate keys for development/production
- 👥 Check group permissions
- 📊 Regularly backup your database

## 🐛 Troubleshooting

### Common issues

**❌ Telegram group ID format**
```bash
TELEGRAM_GROUP_ID=-1001234567890  # ✅ Correct (negative!)
TELEGRAM_GROUP_ID=1234567890      # ❌ Incorrect (positive)
```

**❌ Nostr key format**
```bash
NOSTR_PRIVATE_KEY=nsec1...     # ✅ Correct (nsec1 prefix)
NOSTR_DM_RECIPIENT=npub1...    # ✅ Correct (npub1 prefix)
```

**❌ Database locked**
```bash
# If you get "database is locked" errors:
# 1. Stop all bridge instances
# 2. Delete bridge.db
# 3. Restart the bridge
```

**❌ Messages not forwarded**
```bash
# Check logs:
RUST_LOG=debug cargo run

# Verify:
# 1. Bot has read permissions in Telegram group
# 2. Nostr relays are reachable
# 3. Recipient pubkey is correct
# 4. Database is writable
```

## 🔄 Migration from previous version

If you're upgrading from the group-based version:

1. **Backup your `.env`**
2. **Update `.env`**:
   - Rename `NOSTR_PUBLIC_KEY` → `NOSTR_DM_RECIPIENT`
   - Change `ENCRYPTION_TYPE=nip17` → `ENCRYPTION_TYPE=nip04`
   - Add `DATABASE_PATH=./bridge.db` (optional)
3. **Remove group-specific settings** (unless you need legacy group support)
4. **Restart the bridge**

## 🚧 Roadmap

- [ ] **NIP-17 support** (modern encrypted DMs)
- [ ] **Reply support** (Telegram ↔ Nostr)
- [ ] **Media support** (images, files)
- [ ] **Multi-user support** (multiple Nostr recipients)
- [ ] **Web UI** for configuration
- [ ] **Docker support**

## 📝 License

MIT – See [LICENSE](LICENSE)

## 💬 Support

- 🐛 **Issues**: [GitHub Issues](https://github.com/Walpurga03/nostr-telegram-bridge/issues)
- 🐾 **Nostr**: `npub192jd2dxlqwfnemzz8hsk77z2rn4de3thelw6suvtvqsl79d0udysxzuswy`

---

## 🙏 Support & Donate

If you like this project and want to say thank you, you can support the development via:

- ⚡ **Lightning**: `aldo.barazutti@walletofsatoshi.com`
- ⚡ **Nostr zap**: [npub192jd2dxlqwfnemzz8hsk77z2rn4de3thelw6suvtvqsl79d0udysxzuswy]

Thank you for your support! 🚀

---

**💡 Tip**: This bridge is designed for **one-to-one communication** between a Telegram group and a single Nostr user via DMs. For group-to-group communication, use the legacy `ENCRYPTION_TYPE=group` mode.
