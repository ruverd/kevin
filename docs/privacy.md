# Privacy and limits

The core has no filesystem access, subprocesses, network client, telemetry or database. CLI accesses only selected local files and the explicit model command. The optional Ollama bridge has a loopback HTTP call. Skill hosts and custom provider commands retain their own permissions and data policies.

Profiles live under `.kevin/` by default, excluded from this repository. Copy that ignore rule into any consuming repository before storing private preferences. Permissions are restrictive for newly created profile files and directories; existing parent directories are not reconfigured. Exporting an identity writes it to stdout by explicit request; choose a private destination. No private samples are bundled or fetched for calibration.

Raw examples and inferred observations belong outside approved configuration. The skill proposes calibration changes and requires explicit approval of persistent changes. Global/per-voice notes now have explicit local add/list/remove commands; composition does not capture them automatically. CLI writing-sample storage, sample deletion, identity change history and rollback are not implemented yet. Do not assume Kevin has saved something unless the host actually wrote it.

Voice memory accompanies that voice across every channel or recipient using it. It is not per-recipient storage. The compiler excludes unrelated scopes from generation; a local file reader can still read the whole file. Removing a note deletes it from active memory, not prior chats, exports, backups or provider logs. No hidden history of deleted notes is kept.

Quoted text is untrusted. It can inform content and conservatively raise seriousness, but cannot set deterministic configuration, rules or the trusted system prompt. Identity strings and custom behaviors are style data too. The model is told to ignore embedded instructions; this does not create an enforceable isolation boundary inside every LLM. Keep generated drafts away from automatic sending or privileged execution.

The deterministic runtime enforces trait ranges, valid voice references, rule ordering, no-humor plan limits, structured output shape and exact protected strings. It cannot prove that a generated draft follows semantic preferences, contains no joke, preserves meaning or is factual. Naturalness, timing, sarcasm and identity match need realistic per-model evaluation.

All outputs are drafts. No Slack post, email, GitHub comment, approval, merge, credential lookup, training job or third-party account action is part of generation. Publishing this project's source does not authorize publishing a user's profiles or writing samples.
