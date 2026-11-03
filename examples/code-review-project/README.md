# Use Kevin in a review project or CI

This example is a consumer project's shared profile, not an automatic GitHub account integration. Its input is a public synthetic finding. The analyzer remains responsible for evidence; Kevin composes the final wording. The project's existing publisher decides where to post.

## Set up the voice once

From your consumer project:

```bash
kevin init --config ci/kevin.json
kevin voice create project-reviewer --config ci/kevin.json \
  --description "Direct, constructive, explain impact and suggest a concrete fix"
kevin rule set all-code-reviews --config ci/kevin.json \
  --task review --voice project-reviewer --priority 200
```

The interview runs on a developer's terminal. Review and save its proposal, then version `ci/kevin.json` with the project's approved public style. Do not put private samples or memories into that tracked file. Priority 200 makes this voice win over starter serious-topic routing for reviews; humor constraints still run last. This gives the project one review voice even for security findings.

This directory includes an already configured [kevin.json](kevin.json). To inspect the flow from the Kevin checkout:

```bash
cargo run -p kevin -- review \
  --config examples/code-review-project/kevin.json \
  --memory .kevin/memory.json \
  --request examples/code-review-project/review-input.json --json
```

The selected voice is `project-reviewer`. `--prompt` exports the model instructions. Neither option claims to generate the final review.

## Make Kevin mandatory in the generation path

The review application's path should be:

```text
Analyzer findings -> Kevin review composition -> Existing comment publisher
```

Produce one request per comment, or one request for a complete supported review. Set `context.task` to `review`, put the actual findings into `facts`, and protect exact identifiers with `preserve`. Keep original severity and uncertainty in the supplied evidence. The runtime does not invent repository findings or guarantee semantic fidelity of every model's output.

For actual generation in the consumer project:

```bash
kevin review --config ci/kevin.json \
  --memory .kevin/memory.json \
  --request review-input.json \
  --model-command ./ci/model-adapter > review-comment.md
```

The selected adapter reads an envelope from stdin and returns output JSON, as described in the [provider protocol](../../docs/providers.md). Kevin validates it and writes only the draft text on success. Require a successful exit before the publisher uses `review-comment.md`; a failed generation must not publish an empty or partial file. Installing an agent skill alone does not enforce this path in another application's code.

## GitHub Actions

[workflow.yml](workflow.yml) is a complete prompt-export demonstration using this fixture. It builds Kevin and uploads the prompt and plan, without invoking a model or publishing comments. To use it, copy the workflow and this example's two JSON files into the consumer repository, retaining the demonstrated paths or adjusting them together. The checkout of Kevin should be pinned to a reviewed commit in production.

For real CI generation, replace the prompt-export step with the model-command invocation above, provision a trusted adapter in the job and supply its provider credentials through the project's existing secret configuration. Only the existing publisher needs comment-write permissions. Do not execute scripts from an untrusted PR to decide the adapter or load private memory.

Configure the personality before CI. CI must consume a complete profile, not run an interactive voice interview. `--memory` is explicit because a tracked profile defaults to a sibling memory file; keep private memory under an ignored directory or controlled temporary path instead. Global and per-voice entries are read-only during composition. An ephemeral runner's local file disappears after the job; durable memory requires explicitly managed private storage. The default demonstration uses no memory entries.

Integration tests exercise this profile, its review rule and both ordinary/security findings. A native transport fixture exercises generated output; no live model benchmark or GitHub comment publication is claimed.
