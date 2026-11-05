# Onboarding a client to mailman

mailman is a single static binary speaking IMAP/SMTP (and OAuth) — no daemon, no
database to run. Onboarding is: install → one TOML account → check. Secrets never
live in the TOML: each credential is a `*.command` the binary runs at use time,
so passwords stay in the macOS Keychain, `pass`, 1Password, or any helper.

## 1. Install

```sh
curl -sSL https://raw.githubusercontent.com/memjar/mailman/master/install.sh | PREFIX=~/.local sh
```

Picks the right `mailman.<target>.tgz` from the latest release; checksums are on
the release page (`SHA256SUMS`). Or build: `cargo install --locked --git
https://github.com/memjar/mailman.git`.

## 2. Store the client's secret

macOS (recommended):

```sh
security add-generic-password -s mailman -a <ACCOUNT-KEY> -w   # prompts, no history
```

Linux with pass: `pass insert mailman/<ACCOUNT-KEY>`.

For an iCloud account the password must be an **app-specific password** from
https://id.apple.com (4 segments: `xxxx-xxxx-xxxx-xxxx`). Gmail likewise
requires an app password (2FA on). OAuth servers (Stalwart, Keycloak, Entra,
Google) work through `sasl.xoauth2.token.command` — any helper that prints a
fresh access token on stdout, e.g. a 5-line script calling the refresh endpoint.

## 3. Write `~/.config/mailman/config.toml`

One `[accounts.<id>]` block per mailbox; `default = true` marks the fallback.

### iCloud

```toml
[accounts.client]
name = "Client Name"
email = "user@icloud.com"
default = true

imap.server = "imaps://imap.mail.me.com:993"
imap.sasl.plain.username = "user@icloud.com"
imap.sasl.plain.password.command = "security find-generic-password -s mailman -a client -w"

smtp.server = "smtp://smtp.mail.me.com:587"
smtp.starttls = true
smtp.sasl.plain.username = "user@icloud.com"
smtp.sasl.plain.password.command = "security find-generic-password -s mailman -a client -w"

mailbox.alias.sent = "Sent Messages"
```

### Gmail (IMAP/SMTP with app password)

```toml
[accounts.gmail]
email = "client@gmail.com"

imap.server = "imaps://imap.gmail.com:993"
imap.sasl.plain.username = "client@gmail.com"
imap.sasl.plain.password.command = "security find-generic-password -s mailman -a gmail -w"

smtp.server = "smtps://smtp.gmail.com:465"
smtp.sasl.plain.username = "client@gmail.com"
smtp.sasl.plain.password.command = "security find-generic-password -s mailman -a gmail -w"
```

### Any standard server (Stalwart / Fastmail / Posteo / your own)

Replace host/port with the server's; `smtps://` for implicit TLS on 465,
`smtp://…:587` + `smtp.starttls = true` for submission. See README.md
"Providers" for per-host notes (Fastmail, Posteo, Proton via Bridge, Outlook
via XOAUTH2).

### OAuth (token helper)

```toml
imap.sasl.xoauth2.username = "user@host"
imap.sasl.xoauth2.token.command = "/path/to/token-helper"
```

The helper prints one access token per invocation; refresh lives wherever the
client keeps secrets (Keychain shown above).

## 4. Verify

```sh
mailman account check                 # every backend: OK/FAIL per line
mailman --json envelope list          # real messages = done
```

`account check` exits 0 only when all configured backends authenticate — the
one command a support desk needs.

## 5. Handing it to an agent / automation

- `--json` on every command; failures are JSON error objects on stdout.
- `mailman repl --json` keeps one IMAP session warm (piped lines, one answer each).
- `mailman mcp` speaks Model Context Protocol over stdio (8 reading tools by
  default; `--allow-write` / `--allow-send` widen it) — drop it into any MCP
  client config as `command: mailman, args: ["mcp"]`.
