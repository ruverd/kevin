# Rust migration, version 0.2

Kevin now has one native runtime in Rust: `kevin-core` for decisions and `kevin` for CLI, storage and adapters. The portable skill stays Markdown. There is no parallel Node runtime or mandatory JavaScript toolchain.

## Preserved contracts

First-use defaults now include the requested `tony-stark` voice and 11 developer voices. Ordinary first use needs no `init`; explicit edits persist a profile. Existing saved profiles are not overwritten. These are intentional product changes requested alongside migration.

Existing JSON v1 profiles, identity objects, memory files, voice definitions, routing rules and composition envelopes remain supported. Command names and flags remain available. A project's existing `--model-command` executable continues to receive JSON stdin and produce JSON stdout; it can be written in any language.

For a given saved profile, the deterministic rules, priority ties, language variants, humor limits, inheritance, blends, defaults and memory scope filtering retain their original behavior. Input lengths keep the original UTF-16 convention. Unknown fields, unsupported versions and explicit nulls remain errors. Memory and draft text keep the original trimming behavior. Profile and memory edits use validated atomic replacement; Unix private files/directories retain 0600/0700 creation permissions.

The JavaScript import API is replaced by Rust's `kevin-core` crate. Non-Rust applications can retain the CLI protocol. `kevin eval` replaces the npm evaluation command; `kevin ollama` replaces the Node bridge. `doctor` reports the native runtime instead of a Node version. Error wording and JSON formatting can differ without changing the data contract.

## Test preservation

All 36 original cases were ported as native tests, with the original behavioral assertions against the original v1 profile; first-use assertions reflect the newly requested starter defaults:

| Original suite | Cases | Rust location |
| --- | ---: | --- |
| Core | 15 | `crates/kevin-core/tests/core.rs` |
| Scoped routing | 5 | `crates/kevin-core/tests/scopes.rs` |
| Memory | 4 | `crates/kevin-core/tests/scopes.rs` |
| Voice design | 4 | `crates/kevin-core/tests/design.rs` |
| CLI and provider | 7 | `crates/kevin-cli/tests/cli.rs` |
| Ollama protocol | 1 | `crates/kevin-cli/tests/ollama.rs` |

Before removing TypeScript, its original 36 tests and 30 routing scenarios passed. The original default profile and 15 scenarios are retained independently of the new starter pack. The TypeScript implementation at commit `bbe7ace` generated frozen public fixtures: 34 full composition envelopes and five interview states. Rust tests compare every field, treating equivalent JSON numeric representations equally. Original instructions, default profile and interview questions are retained as embedded assets.

Additional native tests cover explicit null rejection, Unicode boundaries, a relocated executable installing its embedded skill, optional evaluation records, private file permissions, malformed-memory preservation, provider deadlines, invalid loopback ports and a consumer review project's fixed voice. The original 15 routing scenarios run automatically and with a forced voice, giving 30 evaluation cases.

Subprocess tests compile a tiny native transport fixture with the available Rust compiler. The loopback server is a protocol fixture, not a live Ollama model. No model quality, cross-model compliance or actual GitHub comment publication is inferred from these checks.

## Installation and integrations

```bash
cargo install --path crates/kevin-cli --locked
kevin doctor
kevin eval
```

Existing project-local profiles need no migration command. Back up private configuration before replacing your installed executable. Agent-only skill use needs no runtime. The native installer embeds all references and works outside a source checkout.

A binary is built for a particular OS/architecture and may require compatible system libraries. CI tests Linux, macOS and Windows and builds native binary artifacts; this does not imply one executable runs on every system. The source toolchain and dependencies are tracked with `rust-toolchain.toml` and `Cargo.lock`.

See the [consumer project example](../examples/code-review-project/README.md) for routing every review through one voice, including CI. Memory remains explicit and local; ephemeral CI files do not become a durable backend. No new paid service, telemetry, database or automatic model call was added.
