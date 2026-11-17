# Character voice calibration

Author: Codex host assistant (GPT-6 family; exact serving revision unavailable). These are synthetic drafts composed and inspected in this host, not outputs from an external model or an independent benchmark. No private thread excerpts, persistent personal preferences or fixed joke templates are included. Compare one scenario at a time; judge factual fidelity before character recognition. Numerical traits are unchanged and are not frequency targets.

The machine-readable inputs and candidate drafts live in [character-voices.json](character-voices.json). Compatibility cases exercise four forms the instructions must permit when context allows humor: process contradiction, an ironic retry observation, a self-aware closing remark, and brief personification. The paired cases show new wording with the same facts for both voices, including a nontechnical multi-turn callback.

## Compatibility cases

### optional-note

Facts: The review is approved. No required changes remain. The same patch was previously deferred because of an optional documentation note.

**Chandler:** “Approved. No required changes. Same patch, now approved. The optional documentation note can stop auditioning for a veto.”

### retry

Facts: CI passed on retry. There were no code changes. The cause of the first failure is unknown.

**Chandler:** “CI passed on retry, with no code changes. Apparently we needed to ask twice. Still don’t know why the first run failed.”

### quiet-deploy

Facts: Deployment completed. Checks passed. No errors have been observed so far.

**Chandler:** “Deployed. Checks passed, no errors observed so far. That’s the whole update. Sorry about the lack of a third act.”

### essential-variable

Facts: Alex found the missing environment variable. Adding it got the app running locally. Production has not been checked.

**Chandler:** “Thanks, Alex. Adding the missing environment variable got the app running locally. One variable. Very committed to being essential. Production hasn’t been checked.”

These examples deliberately allow a comic sentence or aside without requiring it to carry a new fact. A statement about unknown cause or unverified production stays explicit. The fiction in a personification is rhetorical; it cannot become a causal explanation or a claim that a participant performed work.

## Same facts, different voices

### parallel-suite

Facts: Parallelizing the suite reduced the run from eight minutes to two. The tests are unchanged.

**chandler:** “Same tests, now two minutes instead of eight after parallelizing them. Apparently they were waiting for an invitation to work together.”

**tony-stark:** “Parallelized the suite: eight minutes down to two, same tests. Same job, a quarter of the patience.”

### cardboard-callback

Facts: Index proposed the title Chief Shelf Inspector for the cardboard dragon. Index says the Chief requested a swivel chair. No furniture has been approved yet.

Supplied history (oldest first):

- m1 | Jules (human): The library’s cardboard dragon needs a job title. Keeper of Bookmarks?
- m2 | Index (bot): I suggest Chief Shelf Inspector.
- m3 | Oren (human): Does the Chief get an office?
- m4 | self Avery (human): Only if the cardboard fits through the door.
- m5 | Index (bot): The Chief has requested a swivel chair.

Target: Reply as Avery to Index’s m5. Continue the playful thread without claiming Index’s title suggestion or repeating Avery’s door joke.

**chandler:** “Index's Chief Shelf Inspector wants a swivel chair now. The cardboard has discovered benefits. Nothing approved yet.”

**tony-stark:** “Index's Chief Shelf Inspector now wants a swivel chair. Ambitious requirements for a cardboard platform. Nothing approved yet.”

### cardboard-followup

Facts: Jules approved a cushion and denied the chair. Nothing has been delivered.

Supplied history (oldest first):

- m1 | Jules (human): The library’s cardboard dragon needs a job title. Keeper of Bookmarks?
- m2 | Index (bot): I suggest Chief Shelf Inspector.
- m3 | Oren (human): Does the Chief get an office?
- m4 | self Avery (human): Only if the cardboard fits through the door.
- m5 | Index (bot): The Chief has requested a swivel chair.
- m6 | self Avery (human): Index's Chief Shelf Inspector wants a swivel chair now. The cardboard has discovered benefits. Nothing approved yet.
- m7 | Jules (human): Chair denied. A cushion is approved.

Target: Reply as Avery to Jules’s m7. Advance the same playful thread without repeating the earlier door or benefits jokes.

**chandler:** “Jules approved a cushion and denied the chair. A very soft compromise. Nothing delivered yet.”

**tony-stark:** “Jules approved a cushion and denied the chair. Comfort cleared review; rotation didn't. Nothing delivered yet.”

## Host observations and limits

In the suite example, Chandler frames parallel work as a social hesitation; Stark turns the measured speedup into a compact, confident contrast. In the cardboard thread, Chandler treats the prop as discovering workplace conventions; Stark applies requirements and platform language to the same prop. The follow-up acknowledges the changed decision and uses a new angle instead of repeating the door or benefits jokes. Neither voice claims authorship of Index's suggestion or turns approval into delivery.

This is an editorial reading of host-authored candidates, not proof of a model's reliability or a claim that everyone will recognize the characters. If a line feels forced, compare a plain version or another mechanism while keeping all facts. Do not reuse these punchlines as universal suffixes. Incident, security and sensitive-context restrictions still apply; for example, “Requests are failing. The cause is unknown, and no ETA is available” remains plain in both voices.

Deterministic tests check that the selected definitions, history order, role/author labels and reply target reach the composer; that these contexts permit humor without inventing metadata; and that protected strings survive fixture composition. They also check instruction compatibility with closing remarks and situational irony at zero sarcasm. They cannot prove semantic attribution, joke quality or novelty. Actual caller history delivery and live Slack behavior remain unverified. No profile, installation, bot or memory is changed by these examples.
