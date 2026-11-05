# Driving mailman from Operator

mailman is built for programmatic use — three surfaces, pick by need:

## 1. One-shot commands (simplest)
Every command speaks JSON and never prompts when stdin isn't a TTY:

```bash
mailman --json envelope list -m inbox          # list envelopes as JSON
mailman --json message read 42                 # one message
mailman --json email send < rfc822.txt         # send from a file
```

Parse with `jq`. Exit code is non-zero on failure; `--json` errors are JSON
objects on stdout, so one parser handles both paths.

## 2. `mailman repl` — one warm process, many commands
Connects IMAP once; each stdin line is a command without the binary name:

```bash
printf 'envelope list\nmessage read 42\nquit\n' | mailman repl --json
```

One JSON value per line, failures are error objects. Use this when Operator
runs bursts of mail commands (inbox triage loops, cron digests) — it avoids a
fresh IMAP login per call.

## 3. `mailman mcp` — native MCP server (Model Context Protocol, stdio)
Exposes the shared API as tools: `mailbox_list`, `envelope_list`,
`envelope_search`, `message_read`, `attachment_list`, the composers, etc.
Tool arguments are the CLI flags spelled the same way; answers match `--json`.

Wire into Operator's `~/.axe/operator/settings.json`:

```json
{
  "mcp": {
    "servers": {
      "mailman": { "command": "mailman", "args": ["mcp"] }
    }
  }
}
```

(Confirm the exact key shape against `grep -rn "mcpServers\|mcp" ~/.axe/operator-run/src/`
before editing — the live schema is the source of truth.)

## Non-interactive rules (keychain-safe)
- Secrets come from `*.password.command` in the config (e.g.
  `security find-generic-password -s mailman -a <user> -w`), never from the
  file itself and never from a prompt. No TTY, no hang.
- Config: `~/.config/mailman/config.toml` (template in this repo's README
  "iCloud Mail" section; iCloud needs an app-specific password and
  `mailbox.alias.sent = "Sent Messages"`).
- `-c <PATH>` overrides the config path; `;`/`:` separates multiple.
- Default trimmed build has imap/smtp/maildir only — no jmap/gmail/msgraph
  backends; commands for those are absent, not failing.

## Telegram / automation hook
`--json` + `jq` is the whole contract: a cron job can run
`mailman --json envelope list -m inbox | jq -r '.[].subject'` and hand the
output to Operator's telegram delivery. Keep it a shell pipeline; mailman
needs no knowledge of Telegram.
