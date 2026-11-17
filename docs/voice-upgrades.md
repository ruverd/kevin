# Updating character voices

The runtime's canonical starter definitions live in `assets/default-config.json`. `kevin-core::default_config()` embeds that file. With no saved profile, the CLI uses those definitions; `kevin init` writes a snapshot into a new profile. `resolve_voice` combines the selected profile's inheritance chain, and the composition envelope carries its descriptions, behaviors and avoid list to the generator.

The portable skill's `references/voices.md` mirrors the Chandler and Tony Stark cards. A regression test checks their descriptions, behaviors and avoid lists against the canonical JSON. The native installer embeds the skill and all its references; package tests check the installed copies. Runtime composition instructions and the direct skill both permit a contextual final sentence or aside, distinguish situational irony from sarcasm and require deliberate execution of the selected personality.

## Existing profiles retain their definitions

Replacing the binary or running `kevin skill update` does **not** refresh `.kevin/config.json`, another explicitly selected config, or hosted-bot configuration. A saved `chandler` or `tony-stark` definition remains authoritative even if the name matches a bundled voice. This protects customizations, but also means older definitions can hide the personality improvements.

To prepare an upgrade, inspect the effective definitions with `kevin voices --json` and the exact config path used by the caller (`--config <path>` when needed). Compare only the two relevant voice objects and their parent definitions against the new starter definitions. Review description, behaviors and avoid lists, not just numeric traits. Preserve identity, languages, routing/defaults, custom edits and memory. Preview a focused change before applying it with explicit user authorization. Do not run `init` over an existing profile or replace the whole profile to update two voices.

There is no automatic profile migration in this change. No private profile or installation is changed by the tests or calibration examples. Updating local skill files also does not update another host's instructions or a hosted bot. Inspect those layers separately for old humor prohibitions before claiming end-to-end behavior has changed.

## Verify delivery separately from voice quality

Compare compiled plans using synthetic structured requests, including omitted audience/stakes metadata. Check the selected voice, effective behavior text, humor mode, sarcasm limit and preserved facts. A successful comparison proves configuration delivery, not recognizable humor in model output.

Use the synthetic comparisons in `evals/humor-calibration.md` for human calibration. They were composed in the Codex host; no external model was called. Choose one scenario at a time. A model-quality claim needs outputs from the actual host/model and human assessment; a live Slack or bot claim also needs verification of that flow.
