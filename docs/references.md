# Reference study

Inspected 2026-11-03. This study compares concrete instructions in the repositories, not only project descriptions or directory listings. Claims are scoped to the inspected versions. Commit links below pin the observations. Kevin's prompts and source are original; no third-party prompt text is vendored.

| Reference | Observed mechanism | Decision for Kevin |
| --- | --- | --- |
| [HumanInk](https://github.com/sirambrosio/humanink/blob/3ac3e227f7c081c4871e9a5935b79adca7cfd917/SKILL.md) | Context/severity modes, style samples, pattern scoring, second audit | Use sample-based identity and a final check. Do not present a heuristic AI score or guessed source model as a measurement. |
| [Humanizer](https://github.com/hannsxpeter/humanizer/blob/e8f1248a7d4919689c1f942e356ed95b102d9f07/SKILL.md) | Voice discovery including VOICE.md; voice matching before cleanup; restraint against invented claims | Preserve identity before style cleanup. Use only the explicit profile or conventional current-project path, avoiding broad parent searches. |
| [Talk Skill](https://github.com/DeadAlmighty/talk-skill/blob/e48e4773422875f51868cf53a5172c38642fddc2/SKILL.md) | Short conversation, energy matching, humor cues and natural language | Adapt to supplied cues and conversational purpose. Do not adopt instructions to pretend to be a human or confidently read hidden emotions. |
| [Work Humanizer](https://github.com/phuryn/work-humanizer/blob/0a9065791d021f3dda695f3887cb2ae4c0eacbcf/skills/work-humanizer/SKILL.md) | Fact audit including certainty, commitments, reasons and actors; interview mode for missing writer-specific substance | Protect intent, evidence and qualifications. Operational replies with supplied facts should not trigger a long interview. |
| [Unslop with voice training](https://github.com/theclaymethod/unslop/blob/17ed39c9d0b522f44190ff0c6233867eadee192a/SKILL.md) | Command routing for teach/mimic/cleanup/rewrite; layered profiles and approval-led voice building in supporting flows | Separate composing, learning and persistence. Load calibration detail only when requested. |
| [MYNAH](https://github.com/Percona-Lab/MYNAH/blob/470c63b7ca0a3a66477c3072ab8680d19cb11371/skills/mynah/SKILL.md) | LEARN/COMPOSE modes; distinct audience/channel profiles and language variants; persistent-memory backends | Retain context and language variants, without depending on its memory backend or duplicating identity into every voice. |
| [Respond to Message](https://github.com/mostafa-drz/claude-skills/blob/e289cedeca6f9d9d2fdd26ab43abc7ab3141c2cb/code/respond-to-message/SKILL.md) | Tone profile plus platform references; reply refinement; clipboard-first macOS workflow | Use platform-specific composition. Do not require hardcoded home paths, `pbcopy`, or one host's tool names. |
| [n8n human-like code review](https://github.com/n8n-io/n8n/blob/5339f0c988dd2fc89040cc5150bd3b991a4c6198/.agents/skills/human-like-code-review/SKILL.md) | Context before findings, concrete consequences/fixes, line-number discipline, short conversational comments | Compose supported findings precisely. Do not reduce proven severity into a soft question or assume its repository-specific tools and workflow. |
| [Comment Writer](https://github.com/Gentleman-Programming/gentle-ai/blob/7d3528877d987765e4e5c504ed6049c3da7c4f4c/internal/assets/skills/comment-writer/SKILL.md) | Useful point first, concise collaboration, technical rationale and target-language matching | Start with actionable content, retain necessary reasoning, adapt length to task. Do not make one universal comment formula or punctuation rule compulsory. |

## Comparison with the locally supplied Unslop

The local `unslop` skill inspected in the development environment contains a shared dry, understated voice, optional one-line humor with context guardrails, and general prose cleanup. Its displayed instructions do not contain the external project's `teach` or `mimic` command flows. Those projects share a name but should not be treated as interchangeable versions.

That local skill already contributes useful restraint and fact-based humor. Kevin's additional mechanism is an approved identity with language variants, independently configured voices, explicit context and deterministic routing. Cleanup remains part of composition instead of becoming the entire product. Kevin's selected style governs the communication draft; unrelated host-wide defaults must not silently replace the user's explicit task preferences.

## Scope of conclusions

The repositories provide overlapping pieces. This inspection does not establish that no future version supports Kevin's full scope, and it does not measure which project's generated prose is best. Local source and tests can establish routing and protocol behavior; subjective quality needs comparable real model runs.

Repository licenses vary, and individual files can have different terms. Linking and discussing a mechanism is not permission to relicense upstream prompts. Any future reuse must preserve the relevant notices and satisfy the exact file's license.
