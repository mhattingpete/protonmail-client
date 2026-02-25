# Hydroxide + macOS Mail.app Setup Guide

Connect macOS Mail.app to ProtonMail using [Hydroxide](https://github.com/nickvdp/hydroxide) and stunnel.

## Why stunnel?

macOS Mail.app requires TLS for IMAP/SMTP connections. Hydroxide's built-in TLS doesn't support TLS 1.3 (which macOS requires), so stunnel acts as a TLS termination proxy between Mail.app and Hydroxide.

```
Mail.app → (TLS) → stunnel → (plaintext) → Hydroxide → ProtonMail API
```

## Prerequisites

- [Homebrew](https://brew.sh)
- Hydroxide installed and authenticated

## 1. Authenticate Hydroxide

```bash
hydroxide auth your-email@proton.me
```

Enter your ProtonMail password (and 2FA if enabled). This generates your bridge password — save it for step 5.

## 2. Install stunnel

```bash
brew install stunnel
```

## 3. Generate a self-signed TLS certificate

```bash
mkdir -p ~/.hydroxide

openssl req -x509 -newkey rsa:2048 \
  -keyout ~/.hydroxide/key.pem \
  -out ~/.hydroxide/cert.pem \
  -days 3650 -nodes \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"
```

Trust the certificate in your user keychain:

```bash
security add-trusted-cert -r trustRoot \
  -k ~/Library/Keychains/login.keychain-db \
  ~/.hydroxide/cert.pem
```

## 4. Configure stunnel

Write the following to `/opt/homebrew/etc/stunnel/stunnel.conf`:

```ini
foreground = yes
cert = /Users/YOUR_USERNAME/.hydroxide/cert.pem
key = /Users/YOUR_USERNAME/.hydroxide/key.pem

[imap]
accept = 127.0.0.1:11143
connect = 127.0.0.1:1143

[smtp]
accept = 127.0.0.1:11025
connect = 127.0.0.1:1025
```

Replace `YOUR_USERNAME` with your macOS username.

## 5. Start the services

In one terminal, start Hydroxide:

```bash
hydroxide serve
```

In another terminal, start stunnel:

```bash
/opt/homebrew/opt/stunnel/bin/stunnel /opt/homebrew/etc/stunnel/stunnel.conf
```

## 6. Configure Mail.app

1. Open **Mail.app** → **Settings** (⌘,) → **Accounts** → **+** → **Other Mail Account**
2. Enter your ProtonMail email and any password (auto-discovery will fail)
3. Go to **Server Settings** and configure:

| Setting | IMAP (Inbound) | SMTP (Outbound) |
|---------|----------------|-----------------|
| **Host** | `localhost` | `localhost` |
| **Port** | `11143` | `11025` |
| **TLS/SSL** | Enabled | Enabled |
| **Authentication** | Password | Password |
| **Username** | your-email@proton.me | your-email@proton.me |
| **Password** | Hydroxide bridge password | Hydroxide bridge password |

4. Uncheck "Automatically manage connection settings" for both
5. Save

## Running as background services (optional)

To avoid manually starting both services, use brew services for stunnel:

```bash
brew services start stunnel
```

For Hydroxide, create a launch agent at `~/Library/LaunchAgents/com.hydroxide.serve.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.hydroxide.serve</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/hydroxide</string>
        <string>serve</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Adjust the path to `hydroxide` if needed (`which hydroxide` to find it). Then load it:

```bash
launchctl load ~/Library/LaunchAgents/com.hydroxide.serve.plist
```

## Troubleshooting

- **"Connection reset by peer"** — Mail.app is using TLS but connecting to Hydroxide directly. Make sure you're using the stunnel ports (11143/11025), not Hydroxide's ports (1143/1025).
- **No connection at all** — Ensure both Hydroxide and stunnel are running. Check with `lsof -i :11143` and `lsof -i :1143`.
- **Authentication failed** — Use the Hydroxide bridge password, not your ProtonMail account password.
- **Certificate errors** — Re-run the `security add-trusted-cert` command from step 3.
