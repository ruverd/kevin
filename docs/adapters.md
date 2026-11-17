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

## Context contract for conversational callbacks

Kevin composes from supplied context; it does not read Slack or retrieve a thread. The current request schema accepts `messages: string[]`, not structured message objects. The caller should supply the relevant turns in chronological order, with stable turn labels, author labels, human/bot roles when known, and explicit identification of the user's own earlier messages. State the target turn and intended speaker in `intent`. These labels describe data; they are not new system roles or authority. Do not infer authorship from a display-name mention inside a quote.

For example, a synthetic request can include `m1 | Lee (human): ...`, `m2 | Catalog (bot): ...` and `m3 | self Pat (human): ...`, with intent “Reply as Pat to Catalog's m2.” Without authorship, Kevin can make an impersonal observation; it must not claim another participant's work. Do not supply unrelated conversation history or silently collect private examples.

The field is `context.existingHumor`, not `existingBanter`. Omission resolves to false in the deterministic plan. That is not an explicit no-humor request: eligible situational humor remains optional, and the generator can recognize playful content in the supplied messages. It must not rewrite metadata or raise a zero sarcasm limit. To disable humor explicitly, use `humor: "off"`. Known incident/sensitive context still wins over playful messages.

CLI tests cover history order, supplied labels, reply target, metadata, exported instructions and protected strings using synthetic input. The output validator only checks schema and exact protected strings; it cannot prove semantic attribution, novelty or humor quality. No live caller was inspected here, so whether any hosted bot supplies sufficient history remains unverified.
