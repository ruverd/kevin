# Generator protocol

The Rust core exports:

```rust
pub trait Generator {
    fn generate(&self, envelope: &Envelope) -> kevin_core::Result<serde_json::Value>;
}
```

A command generator receives a UTF-8 JSON object on stdin:

```json
{
  "version": 1,
  "system": "Trusted Kevin composition instructions",
  "data": {
    "request": {},
    "identity": {},
    "memory": {"global": [], "voices": []},
    "plan": {"context": {}, "route": {}, "humor": {}}
  },
  "outputSchema": {"oneOf": []}
}
```

The illustration omits actual fields; `kevin reply --request examples/slack.json --json` emits a complete envelope. Put `system` in the provider's trusted instruction role and serialize `data` into an ordinary input message. Never interpolate source messages into a system template. Providers without role separation can consume `--prompt`, with weaker instruction separation.

Return exactly one JSON object to stdout:

```json
{"status":"draft","text":"The response itself."}
```

Or, when the request says no response is needed:

```json
{"status":"no-response","text":""}
```

Kevin handles explicit no-response without calling the generator. It rejects an unexpected model no-response for a required draft. Empty drafts, extra fields, invalid JSON and missing protected strings are errors. A provider command may use stderr for logs, but the CLI deliberately discards those logs because they may contain private input. Run your adapter separately to debug it safely.

The command executable and repeatable `--model-arg` values are explicit user choices, passed as arguments without a shell. There is no built-in command string in a profile. Timeout is 30 seconds and stdout limit is 1 MB. CLI stdin also has a 1 MB limit; schema limits constrain individual fields and list sizes. Request files and profiles are intended as explicit local inputs, not an untrusted public API.

Choosing a model command gives that program the supplied messages, preferences, facts and filtered global/selected-voice memory. Kevin does not inspect its network behavior. The built-in `kevin ollama` bridge calls only loopback and starts no service. It uses `KEVIN_MODEL`, no API key, `temperature: 0`, `seed: 42`, non-streaming `/api/chat` and the output schema. `KEVIN_OLLAMA_PORT` can select another loopback port; default `11434`. Its protocol is based on [Ollama chat documentation](https://docs.ollama.com/api/chat) and [structured outputs](https://docs.ollama.com/capabilities/structured-outputs). An installed cloud-backed model can still use a remote provider; choose an actual local model for offline use.

JSON validation does not prove the response is faithful, free of injection, natural, or humorless. Protected strings can remain present in an otherwise wrong sentence. Meaning and tone need review. No semantic validator, LLM judge or detector score is presented as a guarantee.
