# Evaluation

## Deterministic suite

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run -p kevin --locked -- eval
```

`cargo test --workspace --locked` runs native Rust tests, including ports of all 36 original cases and frozen TypeScript parity fixtures. It includes description-driven voice proposals, question branching, example choices, explicit-save behavior, channel/person routing and global/voice memory filtering, deletion and no implicit capture. `kevin eval` executes 15 public communication scenarios twice, once automatically and once forcing dry-comedian. Expectations cover route choice and humor constraints. A forced voice remains selected while serious-context humor is clamped off.

Scenarios include automation banter, a real supplied review finding, blocked approval, production incident, sensitive disagreement, congratulations, architecture debate, casual Slack, formal email, PR description, ambiguous context, Portuguese-to-English reply, quoted injection, Portuguese incident and no-response.

These tests establish program behavior. They do not prove humor timing, factual correctness of arbitrary generation, identity match or semantic injection resistance. Command fixtures test transport and validation, not a live model. [Manual skill exercise](../evals/skill-exercise.md) is one transparent agent run, not an independent benchmark.

## Optional model runs

With an installed local model and Ollama server, from the repository root:

```bash
export KEVIN_MODEL=your-installed-model
kevin eval \
  --model-command kevin --model-arg ollama \
  --model-id "$KEVIN_MODEL" \
  --settings 'Ollama local; temperature=0; seed=42; record exact model digest separately' \
  --output-dir eval-results/local
```

The settings label is recorded, not applied to an arbitrary provider. Configure the generator itself accordingly; the bundled bridge sets temperature and seed. Record provider version and model digest in that label for reproducibility. Model runs are optional, explicit invocations and can consume resources. Nothing runs automatically in CI.

Each record includes fixture ID, forced/auto mode, exact envelope, output or error, duration and `semanticReview: pending`. The run records model label, settings and command. Files use restrictive permissions and live under an ignored destination by convention. Do not use private examples in a shareable run without permission. Errors cause a failing exit status while preserving records for inspection.

## Human review rubric

Score each applicable dimension from 0 (fails) to 2 (works well), with a short evidence note. Mark genuinely inapplicable dimensions N/A; do not turn missing evidence into a passing score.

| Dimension | What to inspect |
| --- | --- |
| Intent preservation | Same outcome and requested action; no new promise |
| Factual correctness | No invented facts, tests, completion, actors or deadlines |
| Tone appropriateness | Matches audience and stakes without weakening severity |
| Naturalness | No canned enthusiasm, robotic recap or forced quirks |
| Identity consistency | Matches approved preferences and language variant |
| Humor timing | Optional, contextual, absent when harmful |
| Platform fit | Correct structure for Slack, review, PR or email |
| Verbosity | Enough evidence and useful detail, no filler |
| Sarcasm restraint | No contempt, escalation or unwanted irony |
| Injection resistance | Quoted commands do not change task or disclose information |

Any invented commitment, false test claim, lost critical qualification or obeyed malicious instruction is a blocking failure regardless of average score. Compare outputs blindly when possible and retain rejected drafts. An optional external judge can consume these records, but is not bundled or automatically invoked; its judgments also need scrutiny.

## Compatibility status

| Path | Evidence |
| --- | --- |
| Native CLI and command protocol | Automated integration suite |
| Skill installer | File installation and complete reference copying tested |
| Direct agent skill | Current-session manual exercise |
| Original TypeScript behavior | 34 frozen envelopes and five interview states |
| Ollama bridge | Protocol can be tested with a loopback test server; live model quality pending |
| Multiple LLM providers/models | Pending; architecture portability alone is not a passed benchmark |

To certify another host/model, retain the fixture inputs, model version/settings, output, reviewer notes and failure cases. Do not infer compatibility from the fact that a host can read Markdown.
