# Dependency mirrors

Every git dependency mailman builds from is a **public mirror under the memjar
organisation**. Nothing in the build graph fetches from a third party's
repository. `cargo build` on a fresh machine resolves only
`github.com/memjar/...` and crates.io.

## Why mirrors

The engine crates (IMAP/SMTP/SASL/Maildir clients, the CLI and config
primitives) come from the upstream project mailman was forked from. We depend
on their behaviour, not on their infrastructure: a mirror means an outage, a
rename or a force-push upstream cannot break our releases or a client's
install. See NOTICE for the attribution and licence terms those crates carry.

## The mapping

| crate | fetched from (pinned rev) |
|---|---|
| `pimalaya-cli` | `memjar/cli` @ `4c9a6b5998555dac338601b1c8de3ce2d15207a9` (v0.3.0) |
| `pimalaya-config` | `memjar/config` @ `7963c5b7bce4e49e93d9bca706fa2ff7d68aadfb` (v0.3.0) |
| `pimalaya-stream` | `memjar/stream` @ `966def319f71dcfa5233a0b09ee728148350023f` (v0.3.0) |
| `io-imap` | `memjar/io-imap` @ `a0d6100626f864c3cae2f75e34587ae7a6557fea` (v0.8.0) |
| `io-smtp` | `memjar/io-smtp` @ `e8f67c167823ecdff156d4a0d8fdcfe0a7d7499c` (v0.6.0) |
| `io-sasl` | `memjar/io-sasl` @ `75d38a8fd09d8376b8f0337c6aa770051417a7b6` (v0.1.0) |
| `io-maildir` | `memjar/io-maildir` @ `7c7aafc3bba6538cd633023eca5356f0def1134d` (v0.3.1) |
| `io-proxy` | `memjar/io-proxy` @ `d80a50f349ea6270769ec37f54661ad797f153b5` (v0.1.0) |
| `ical-rs` | `memjar/ical` @ `99b92736a6a076c72936fac29c9b85a5168e4c2f` |
| `io-http` | `memjar/io-http` @ `e1815307771c1ec65a86b77256d46c33241d3038` |
| `io-jmap` | `memjar/io-jmap` @ `919eab465ab620b323f9191ab8de2f640bc4a1d2` |
| `io-msgraph` | `memjar/io-msgraph` @ `2ae26370aefa671c16316dbad98107634436bce6` |
| `io-pimdir` | `memjar/io-pimdir` @ `0c701447473a03e5cf9effcd0039a49a78419c4d` |
| `mime-rs` | `memjar/mime-rs` @ `d7ea03296a197647a6effeb593a8528b47bdb0ed` |
| `base64-codec` | `memjar/base64-codec` @ `d39f6b90f02abf8eb91ef041b7601320df8def99` |
| `quoted-printable-codec` | `memjar/quoted-printable-codec` @ `d63612b0f9c046717e183e3be534c98f300d913a` |
| `simd-cpu` | `memjar/simd-cpu` @ `f2eb72cff2fafad8ae135e00685bc19f9e80e5cc` |

The first eight are crates.io crates pinned through `[patch.crates-io]`; the
last nine are git sources, four of them pulled transitively by the mirrored
crates, which is why they need `[patch."<source url>"]` — a patch names the
source it redirects, so those table headers keep the upstream URL by design.

## Mirrors do not run CI

Each mirror has GitHub Actions disabled. They are content, not projects: their
workflows reference upstream secrets and would only ever fail. Verification of
this content is mailman's own CI, which builds and tests against the pinned
revisions.

## Adding or bumping a mirror

1. `git clone --mirror https://github.com/<owner>/<repo> <name>.git`
2. `gh repo create memjar/<name> --public` (org repo creation goes through the
   GitHub UI or `gh repo create`; the REST org endpoint needs `admin:org`).
3. Push heads **and** the tags: `git push --mirror` is rejected for refs GitHub
   forbids, so push `refs/heads/*` then `refs/tags/<tag>` individually.
4. Resolve the tag to a commit: `git rev-parse v0.3.0^{commit}`. `rev =` in
   Cargo.toml must be a **commit** sha — an annotated-tag sha is unreachable to
   cargo and fails with `revision ... not found`.
5. Add the `[patch]` entry, run `cargo generate-lockfile`, then
   `cargo build --release && cargo test`.
6. Update this table.

## Guard

`.github/workflows/tests.yml` fails the build if `Cargo.toml` declares a
`git = "https://github.com/(pimalaya|soywod)/..."` fetch target or if
`Cargo.lock` records such a source. That is what keeps this property true as
dependencies move.
