# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed — AXE fork (mailman)

- Forked from an upstream email-CLI checkout as `mailman`: full brand rename, own CI (tests / audit / releases), own logos, own license files (see NOTICE).
- Default features trimmed to `rustls-ring, imap, smtp, maildir, vendored`. The heavier backends (jmap, gmail, msgraph, mbox, m2dir, sieve, pimdir) and the wizard remain available as opt-in features: `cargo build --release --features jmap,gmail,...`. The default binary drops from 14 MB to 6.5 MB.
- Releases publish per-target tarballs (`mailman.<target>.tgz` for aarch64/x86_64 darwin+linux) with `SHA256SUMS`; `install.sh` now resolves them from `github.com/memjar/mailman`.
- `docs/CLIENT-ONBOARDING.md`: wiring mailman for clients on iCloud, Gmail, and any standard IMAP/SMTP/OAuth server (Stalwart, Fastmail, Posteo), secrets kept in the OS keychain or a command helper, never in the config file.


- Added `mailman init --provider icloud|gmail|imap`, a non-interactive template writer printing a ready-to-paste `[accounts.<id>]` block with the Keychain `password.command` pattern.
- Added `docs/CLIENT-ONBOARDING.md` and a Homebrew tap (`memjar/homebrew-mailman`).

### Notes

- This file records AXE's changes only. The pre-fork history of the upstream
  project is not mirrored here; see NOTICE for attribution and license terms.

## [2.2.1] - 2026-11-05

First AXE release: trimmed default features (imap, smtp, maildir, rustls-ring,
vendored), cross-platform release tarballs with SHA256SUMS, own CI (tests,
audit, releases), own branding and license files, `mailman init`.
