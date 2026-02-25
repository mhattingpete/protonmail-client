# ProtonMail Client

A native macOS desktop email client for ProtonMail, built with Tauri, React, and Rust.

## Prerequisites

- [Bun](https://bun.sh) (JS runtime and package manager)
- [Rust](https://rustup.rs) (for the Tauri backend)
- [Hydroxide](https://github.com/nickvdp/hydroxide) (ProtonMail IMAP/SMTP bridge)

## Setup

1. **Clone and install dependencies:**

   ```bash
   git clone https://github.com/mhattingpete/protonmail-client.git
   cd protonmail-client
   bun install
   ```

2. **Authenticate Hydroxide** with your ProtonMail account:

   ```bash
   hydroxide auth your-email@proton.me
   ```

   Enter your password (and 2FA if enabled). Save the generated bridge password.

3. **Start Hydroxide** in the background:

   ```bash
   hydroxide serve
   ```

4. **Run the app in development mode:**

   ```bash
   bun tauri dev
   ```

## Build

Create a production build:

```bash
bun tauri build
```

The bundled app will be in `src-tauri/target/release/bundle/`.

## Hydroxide + macOS Mail.app

If you also want to use ProtonMail with macOS Mail.app (via stunnel for TLS), see [SETUP.md](SETUP.md).
