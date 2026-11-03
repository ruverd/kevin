# Architecture and public contracts

Kevin has one npm package with small internal modules rather than a workspace of separately versioned packages. A provider-independent public API is already available; publishing internal modules separately would add release work without an initial consumer requirement.

| Location | Responsibility |
| --- | --- |
| `packages/core/src/schema.ts` | Runtime schemas and schema-derived TypeScript types |
| `packages/core/src/defaults.ts` | Starter voices and rules |
| `packages/core/src/context.ts` | Supplied context plus conservative seriousness escalation |
| `packages/core/src/router.ts` | Rule ordering, inheritance, named blends, humor constraints |
| `packages/core/src/composer.ts` | Composition envelope, generator interface, output validation |
| `packages/core/src/voice-design.ts` | Validated questionnaire, next gap, example choices and voice proposals |
| `packages/core/src/memory.ts` | Versioned memory schema and selected-voice scope filtering |
| `packages/cli/src/` | Explicit local files and command adapter |
| `skills/kevin/` | Portable instructions with no runtime dependency |
| `evals/` | Public scenarios and optional recorded model runs |

## API

After building, import from `dist/packages/core/src/index.js` in a checkout, or from the package's exported entrypoint when installed:

```ts
import { compile, compose, defaultConfig } from './dist/packages/core/src/index.js';
import type { Envelope, Generator } from './dist/packages/core/src/index.js';

const envelope = compile({
  intent: 'Write one constructive comment.',
  context: { platform: 'github', task: 'review', seriousness: 'ordinary' },
  facts: ['items[0].id throws when items is empty.'],
  preserve: ['items[0].id'],
}, defaultConfig());

// Your adapter implements generate(envelope: Envelope): Promise<unknown>.
// await compose(envelope, generator) validates structured output and protected strings.
```

`compile(request: unknown, config: unknown, memory?: unknown): Envelope` validates external boundaries, resolves context and route, applies humor constraints, selects global/voice memory and returns trusted instructions separately from JSON data. Omitted memory is empty. `compose(envelope, generator): Promise<Output>` does not call the generator when no response is needed. It validates output, rejects a model's unexpected no-response and checks protected exact strings.

An envelope is an object, not a signed authorization token. Applications must keep its trusted fields intact and must never let third-party input substitute `system` or `plan`. Call `compile` at the trust boundary. There are no model tool calls in this protocol.

## Configuration v1

JSON only in the initial runtime. `configSchema` and `requestSchema` are exported Zod schemas. Unknown keys and unsupported versions fail rather than silently disappearing. `kevin init` produces a complete editable configuration; [identity reference](../skills/kevin/references/identity.md) contains a minimal example.

| Field | Default / meaning |
| --- | --- |
| `version` | Required literal `1` |
| `identity.name` | Empty string |
| `identity.primaryLanguage` | `en-US`; explicit context language wins |
| `identity.traits` | directness `.75`, formality `.35`, warmth `.5`, humor `.25`, sarcasm `0`, verbosity `.3` |
| `identity.preferences`, `identity.avoid` | Empty lists; approved style data |
| `identity.languages` | Exact language tag → partial traits and preferences |
| `voices` | Required map of custom or built-in voices |
| `routing.fallback` | `neutral`, which must exist |
| `routing.rules` | Empty unless configured; `init` supplies starter rules |

If `identity.traits` is supplied it must specify all six traits. Voice traits and language traits are partial. Trait values range from `0` to `1`; they are relative guidance, not calibrated personality scores. Identity text preferences are passed to the model. Natural-language prohibitions are not machine-interpreted by the router; use request `humor: "off"` for deterministic disabling.

A voice has required `description`, optional `extends`, and optional partial `traits`, `behaviors`, `avoid`. Voice names use lowercase letters, digits and hyphens, begin with a letter and have at most 64 characters. References must exist, and inheritance cycles fail validation. Blends apply overlays in declared order, with the last explicitly set trait winning. Behaviors accumulate; this is deliberate named composition, not numerical averaging.

Resolved descriptions are included in the model-facing route as style data, including inherited descriptions. They are never inserted into the trusted system instruction.

Rules have `name`, integer `priority` (default `0`), `when`, `voice`, optional partial `overrides`. Conditions are exact equality and AND; omitted fields do not participate. Descending priority, then declaration order, selects the first matching rule. Only that rule applies. A manual voice bypasses automatic routing and rule overrides. Blends and explicit context formality follow; humor constraints always run last.

## Request and context

`intent` is required. Optional `draft`, `messages`, `facts`, `preserve`, `context`, `voice`, `blend`, `humor` describe the communication. `preserve` contains literal strings that must appear unchanged in output, useful for identifiers or an exact caveat. `humor` is `auto` or `off`.

| Context field | Accepted values / default |
| --- | --- |
| `platform` | slack, github, email, chat, docs, unknown / unknown |
| `task` | reply, rewrite, review, pr-description, explain, summarize, draft / reply |
| `audience` | teammate, manager, external, public, unknown / unknown |
| `seriousness` | ordinary, sensitive, critical, unknown / unknown |
| `energy` | calm, playful, tense, unknown / unknown |
| `existingHumor` | Boolean / false |
| `responseNeeded` | Boolean / true |
| `language` | Language tag / identity primary language |
| `formality` | Optional number in [0, 1] |
| `workspace` | Optional explicit workspace identifier, up to 200 characters |
| `channel` | Optional explicit channel identifier, up to 200 characters |
| `recipient` | Optional explicit recipient identifier, up to 200 characters |

Low confidence means platform or seriousness is unknown. This label is an input-completeness indicator, not a measured probability. An exact matching configured rule still applies to supplied fields; otherwise routing uses the default, initially neutral. Unrelated missing metadata does not erase an explicit channel/person assignment. Unknown stakes keep humor off regardless of voice. The analyzer can conservatively escalate seriousness for critical or sensitive English/Portuguese terms in supplied text; it never downgrades critical context. Quoted malicious text can cause a conservative escalation, but cannot change a rule or trusted instruction.

Keyword detection is deliberately limited and can produce false positives, including negated mentions of an incident. Other languages and indirect sensitive situations require explicit context. No thread history, platform, role or emotion is secretly inferred.

## Persistence and migration

CLI uses only the explicitly selected profile, defaulting to `.kevin/config.json` in the current directory. No parent-directory discovery. New files use restrictive permissions; edits write a validated temporary file and atomically rename it. `init` refuses overwrites. Configuration edits assume one local writer; no concurrency coordinator or history store exists in Phase 1.

Version 1 is the initial format. Unsupported versions fail with a validation error. There is no prior format to migrate; future schema changes must add an explicit migration, test fixtures and an inspectable diff, rather than auto-renaming fields. No silent migration runs today.

Identity imports are explicit complete replacements of the identity object; custom voices and routing stay intact. `identity export` exports that object. Back up a profile before manual changes. Automatic history and rollback belong to Phase 2.

CLI rules can be listed with `kevin rules`, created/replaced with `kevin rule set <name> --voice <id> <context flags> [--priority 80]`, and removed by name. Rule names must be unique. Set requires at least one explicit condition. No account lookup or implicit recipient matching occurs. Workspace/channel/recipient are opaque exact strings; a host adapter must supply them from authorized context.

## Scoped memory

`memorySchema` validates `{version: 1, global: MemoryEntry[], voices: Record<voiceId, MemoryEntry[]>}`. Each entry has `id`, `kind` (note/preference), `text`, ISO `createdAt`, and `source: "user"`. Each scope has at most 100 entries of at most 2000 characters. Duplicate IDs within a scope fail validation. The source label records provenance convention, not authenticated authorship.

`selectMemory(memory, selectedVoices)` returns global entries and the deduplicated explicitly selected voice scopes only. Voice inheritance does not inherit memory; a blend explicitly selects each named scope. The compiler filters before serializing an envelope, so unrelated voice entries do not reach the generator. Memory text is untrusted data and cannot change deterministic routing or trusted instructions.

CLI stores memory in a separate private JSON file next to the selected config. Missing files read as empty; malformed files fail without reset. Only explicit `memory add` and `memory remove` write, using validated atomic replacement. Composing never captures notes or updates identity. Current intent/evidence and correctness limits outrank memory. See the portable [memory guide](../skills/kevin/references/memory.md) for host behavior and deletion limits.

## Personality design

`designVoice(description: unknown, answers: unknown): DesignResult` is a pure public API. It returns either `status: "question"` with one next question, recommendation, rationale and options, or `status: "proposal"` with validated voice JSON and illustrative previews. It never saves preferences or calls a model.

`designAnswersSchema` accepts optional `context` (slack/review/email/mixed), `stance` (direct/warm/tactful), `humor` (none/dry/playful), `detail` (short/balanced/deep), `example` (a/b/c) and optional `avoid` list. Question order resolves context before showing context-specific example replies. The local questionnaire deliberately does not pretend to infer semantic traits from the free-form description; that description is retained for model-facing style guidance.

CLI `voice create <name> --description <text>` runs the questionnaire in a terminal or returns the next question as JSON for noninteractive use. `--answers <file>` supplies partial or complete choices. Complete answers show a proposal without saving by default; terminal confirmation or explicit `--save` adds only the voice. Incomplete `--save`, invalid choices and existing voice names fail without changing the profile. The model-native skill can conduct a deeper open-ended gap interview using the same constraints.
