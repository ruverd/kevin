# Agent adapters

The adapter is the same portable skill directory, copied intact. Kevin does not depend on a vendor's memory, environment variable, hidden history or slash-command dispatcher. The CLI is an optional deterministic helper, not a requirement for the skill.

| Host | Filesystem installation | Context available | Output |
| --- | --- | --- | --- |
| Codex | `.agents/skills/kevin` per project | Supplied chat and host-authorized tools | Natural draft, normally `$kevin` or implicit selection |
| Claude Code | `.claude/skills/kevin` per project | Supplied chat and tools allowed by host | Natural draft, normally `/kevin` |
| Cursor | `.cursor/skills/kevin` per project | Depends on version and enabled skills support | Natural draft via selected skill |
| Other agents | Load SKILL.md plus requested references | Only what that host exposes | Natural draft or host-defined response |
| CLI command | Explicit request JSON | No automatic history or account access | Prompt, envelope, or generated draft |

Host versions can change installation discovery and invocation UI. If discovery fails, explicitly load `skills/kevin/SKILL.md` and the relevant referenced files rather than claiming that a slash command must exist. A text-only model can use pasted instructions and a supplied profile. A filesystem-capable agent can read the project-local approved profile. Tools are optional; lack of them must not become fabricated persistence.

The direct skill may infer target language from clear thread evidence, then fall back to the profile. The CLI intentionally uses explicit language or profile primary language, without guessing. Provide language in a request for reproducible routing/composition.

`/kevin --voice=...`, `/kevin calibrate`, and `/kevin voice create <description>` are agent request conventions, not a universal command dispatcher. In Codex, express them as `$kevin ...` or plain instructions. The shell CLI uses `kevin reply --voice ...` and `kevin voice create <name> --description ...`; it does not yet implement identity calibration. Do not advertise unsupported command discovery as an integration test.

Installed skill file copying is integration-tested. This repository does not claim to have launched every host. [Manual exercise](../evals/skill-exercise.md) records one current-agent use; the cross-model matrix is pending.
