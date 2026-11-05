# Contributing guide

Thank you for investing your time in contributing to Mailman CLI.

Whether you are a human or an AI agent, read these in order before touching the code:

3. the inline header documentation in src/main.rs: it is the architecture document of this binary, covering the backends and plumbing, the three command families, and the shared versus protocol-specific surface;
4. the `_axe/` folder, which follows Axe-spec (`spec/` is current truth, `changes/` holds in-flight proposals, `log/` is the dated history), activated by AGENTS.md.

Everything below documents this repository's own standards.

## Where changes belong

Mailman is the CLI front-end of the email stack, a thin shell driving the sans-I/O `io-` crates. Triage before patching, since protocol and storage fixes usually belong in those crates:

- IMAP, JMAP, Gmail, Microsoft Graph and SMTP wire semantics belong in `io-imap`, `io-jmap`, `io-gmail`, `io-msgraph` and `io-smtp`;
- local storage semantics belong in `io-maildir`, `io-m2dir` and `io-mbox`;
- account discovery consumed by the wizard belongs in `io-pim-discovery`;
- the commands, rendering, composition, the wizard and the shared cross-protocol surface live here.

The clap, printer, prompt and spinner primitives come from `pimalaya-cli`, the TOML loader and secret resolution from `pimalaya-config`, the TCP, TLS and SASL plumbing from `pimalaya-stream` (all vendored under the memjar org, see NOTICE). The src/main.rs header maps each backend to its crate.

## Feature matrix

Mailman is a binary, not a layered library, so it has no coroutine/client split. Its cargo features gate the backends (`imap`, `smtp`, `jmap`, `gmail`, `msgraph` and `maildir` on by default, `m2dir`, `mbox` and `pimdir` off), the ManageSieve service protocol (`sieve`), the setup `wizard`, and the TLS provider (`rustls-ring` default, `rustls-aws`, `native-tls`). Build a reduced set to check the feature gates still hold when touching them:

```sh
cargo build --no-default-features --features imap,smtp,rustls-ring
cargo build --no-default-features --features jmap,rustls-ring
cargo test --no-default-features --features sieve,rustls-ring
```

## Dependencies

Cargo.toml patches the vendored crates still ahead of their crates.io release to git. To build against a local checkout, swap the matching `.git` entry for `.path = "../<repo>"`. If cargo reports two versions of a crate, patch every vendored crate that pulls it transitively so the graph converges on the local copies.
