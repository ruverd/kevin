# Contributing

Kevin is small by design: a portable skill, a deterministic core and a CLI. Keep provider clients and filesystem operations out of the core. Prefer an example or configuration change over a new framework.

Use the Rust toolchain in `rust-toolchain.toml`. Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, and `cargo run -p kevin --locked -- eval`. Runtime input changes need schema and boundary tests. Routing changes need a realistic fixture that shows the intended behavior, including forced-voice constraints when relevant. Keep optional model runs outside normal CI.

Do not add private conversations, identity profiles, API keys or generated model logs to commits. Public synthetic fixtures belong in `evals/scenarios.json`; label them as synthetic. Profile examples must contain fictional or explicitly approved information.

Describe changes in observable terms: triggering context, previous behavior, resulting behavior and validation. A prose-style change does not need tests that match exact wording. A correctness constraint needs a meaningful test, not a screenshot or a claim that a model would probably obey.

Original code and documentation use MIT. Check file-specific upstream licenses before importing any code or prompt. The README banner is supplied media outside the software license. Community voice packs should describe reusable traits rather than depend on copied dialogue.
