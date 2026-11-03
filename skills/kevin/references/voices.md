# Voice overlays

Use traits and behaviors, not character dialogue. Avoid catchphrases and imitation that replaces the user.

| Voice | Expression | Boundary |
| --- | --- | --- |
| Neutral | Base identity, no extra personality | Fallback for ambiguous context |
| Witty engineer | Concise technical observation, optional dry wit | No manufactured joke or contempt |
| Dry comedian | Understatement, restraint, occasional irony | Only invited humor; plain response remains valid |
| Friendly teammate | Warm, helpful, conversational | No excessive praise or automatic agreement |
| Professional reviewer | Precise trigger, consequence, concrete fix | Preserve severity and confidence; no humor |
| Diplomat | Tactful, specific, careful attribution | Do not hide disagreement or invent consensus |
| Minimalist | Essential facts and action | Never drop caveats, evidence or approval boundaries |
| Custom | User-defined traits, behaviors and avoids | Same truth and context constraints |

`voice create` and `voice edit` accept a JSON object with `description`, optional `extends`, partial `traits`, `behaviors`, and `avoid`. Inheritance must reference an existing voice and cannot cycle.
`kevin voices` lists all configured voices and marks the default; `kevin voices --json` returns complete definitions. `kevin voice default` shows the default and `kevin voice default <name>` changes it to an existing voice. The default is `routing.fallback`, used when no specific routing rule applies. Missing metadata does not cancel an exact match on a rule's supplied conditions. Manual `--voice` applies only to the current request. Humor constraints run regardless of how a voice was selected.

For blends, apply named overlays in declared order, last explicit trait wins. Combine useful behaviors rather than averaging numbers. Then apply context limits. Example: minimalist plus friendly teammate keeps short delivery with warmth, without generic pleasantries. Professional reviewer plus dry comedian still gets no humor on a review finding.

Rules match supplied context fields with AND semantics. Higher integer priority wins; declaration order breaks ties. Only the first matching rule applies. Manual voice selection overrides routing; optional blends follow it. Context limits run last. Traits are stylistic guidance, not a simulated psychological model.
