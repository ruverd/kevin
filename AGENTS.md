# Repository guidance

Kevin is a portable communication skill with an optional native Rust runtime. Keep model-specific clients, filesystem access and subprocess execution outside `crates/kevin-core`.

Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, and `cargo run -p kevin --locked -- eval` after runtime changes. Keep model evaluations optional and explicit. Do not describe deterministic fixtures as model-quality measurements. Public synthetic inputs are in `evals/scenarios.json`; keep private profiles and samples out of version control.

<!-- JULIUS_START -->
## Julius

Use Julius for code navigation and change investigations in the current repository.
Prefer the `julius_discover` and `julius_call` MCP tools: discover an operation's
schema before calling it. Graph operations build or refresh the local structural
index on demand, without a model key. Keep queries scoped to the current worktree.

CLI fallback: `julius ask "<symbols or question>" --source .`.
Use targeted source searches when Julius is unavailable or misses relevant code.
Repository state lives under `.julius/`; shared learnings live in
`.julius/learnings/`. Do not enable paid/deep analysis or publish learnings
without authorization. Token estimates are not provider measurements.
<!-- JULIUS_END -->
