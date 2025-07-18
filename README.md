[🇩🇪 Deutsch](README.de.md)
# nostr-telegram-bridge

A bridge application that forwards messages from a Telegram group to Nostr. Supports NIP-04, NIP-17, public messages, and Nostr groups (NIP-29).

## Features

- 📱 Telegram group → Nostr forwarding
- 🔒 **NIP-17** (private messages) – default
- 🔐 **NIP-04** (legacy encryption) – compatibility
- 🌐 **Public messages** – no encryption
- 👥 **Nostr groups (NIP-29)** – group chat
- 🔄 Multi-relay support
- 🛑 Graceful shutdown
- ⚙️ Configuration via `.env`

## Requirements

- Rust 1.70+
- Telegram bot token
- Nostr private key
- Nostr public key (only for encrypted messages)
- Telegram group ID
- **For group mode:** Nostr group event ID and group relay

## Installation

```bash
git clone https://github.com/Walpurga03/nostr-telegram-bridge.git
cd nostr-telegram-bridge
cargo build --release
cp .env.example .env
```

## Quickstart

1. Configure `.env` (see below)
2. Start the bridge: `cargo run`
3. Send a message in your Telegram group → It appears on Nostr

## Configuration

Example `.env`:

```env
# Telegram configuration
TELEGRAM_BOT_TOKEN=1234567890:ABCdXXXXXXXXXXXXXXXXXXXXXXx
TELEGRAM_GROUP_ID=-1001XXXXXXXXXXXXx

# Nostr configuration
NOSTR_PRIVATE_KEY=nsec1abcdef...
NOSTR_PUBLIC_KEY=npub1abcdef...  # Only for nip04/nip17

# Relay configuration
NOSTR_RELAYS=wss://relay.damus.io,wss://nos.lol,wss://relay.snort.social

# Encryption type
ENCRYPTION_TYPE=nip17

# Group configuration (only for ENCRYPTION_TYPE=group)
NOSTR_GROUP_EVENT_ID=dde39dbaf95c637ea8XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
NOSTR_GROUP_RELAY=wss://groups.0xchat.com
```

## Encryption types

| Type     | Description                          | Recipient needed | Special config         |
|----------|--------------------------------------|------------------|------------------------|
| `nip17`  | Modern private messages (default)    | ✅               | ❌                     |
| `nip04`  | Legacy encryption (compatibility)    | ✅               | ❌                     |
| `public` | Public messages                      | ❌               | ❌                     |
| `group`  | Nostr groups (NIP-29)                | ❌               | ✅ Event ID + relay    |

## Setup steps

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

### 4. For group mode: Set up a Nostr group
- **Option A**: Use an existing group (e.g. via [0xchat](https://0xchat.com))
- **Option B**: Create a new group

#### Find the group event ID
```bash
# In 0xchat: Group info → Copy event ID
# Format: 64-character hex string
# Example: dde39dbaf95c637ea8785583e4c1a64be0462f3609695592c433ee6697b19815
```

## Usage

```bash
# Development
RUST_LOG=info cargo run

# Production
./target/release/nostr-telegram-bridge

# Debug mode (verbose logs)
RUST_LOG=debug cargo run
```

### Message flow
1. **Telegram**: Send a message in the configured group
2. **Bridge**: Receives and formats the message
3. **Nostr**: Message is forwarded according to `ENCRYPTION_TYPE`

## Security

- ❌ **Never** commit private keys or bot tokens to git
- 🔒 Secure your `.env` file: `chmod 600 .env`
- 🛡️ Use **NIP-17** for best security
- 🔑 Use separate keys for development/production
- 👥 Check group permissions

## Troubleshooting

### Common issues

**❌ Telegram group ID format**
```bash
TELEGRAM_GROUP_ID=-1001234567890  # ✅ Correct (negative!)
TELEGRAM_GROUP_ID=1234567890      # ❌ Incorrect (positive)
```

**❌ Nostr key format**
```bash
NOSTR_PRIVATE_KEY=nsec1...  # ✅ Correct (nsec1 prefix)
NOSTR_PUBLIC_KEY=npub1...   # ✅ Correct (npub1 prefix)
```

**❌ Group permissions**
```bash
# Bot is not authorized in the Nostr group
# Solution: Group admin must grant bot permission
```

**❌ Relay connection**
```bash
# Test group relay
curl -I wss://groups.0xchat.com
# Should return "101 Switching Protocols"
```

## Encryption type comparison

### 🔒 NIP-17 (recommended)
- ✅ Modern cryptography
- ✅ Better metadata protection
- ✅ Protection against timing attacks
- ✅ Future-proof
- ⚠️ Requires newer clients

### 🔐 NIP-04 (legacy)
- ✅ Maximum client compatibility
- ✅ Proven technology
- ⚠️ Older cryptography
- ⚠️ Possible metadata leaks

### 🌐 Public
- ✅ No encryption needed
- ✅ Maximum compatibility
- ✅ Simple setup
- ⚠️ Anyone can read

### 👥 Groups (NIP-29)
- ✅ Group chat functionality
- ✅ No recipient config needed
- ✅ Scalable for many users
- ✅ Moderatable by admins
- ⚠️ Group setup required
- ⚠️ NIP-29 client support needed

## License

MIT – See [LICENSE](LICENSE)

## Support

- 🐛 **Issues**: [GitHub Issues](https://github.com/Walpurga03/nostr-telegram-bridge/issues)
- 🐾 **Nostr**: `npub192jd2dxlqwfnemzz8hsk77z2rn4de3thelw6suvtvqsl79d0udysxzuswy`

---

**💡 Tip:** For getting started, we recommend the **NIP-17 mode** for private messages or **group mode** for community chats.