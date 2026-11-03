// Optional command bridge. Starts no server, downloads no models, sends only to loopback.
import { envelopeSchema } from './provider-contract.mjs';

try {
  const chunks = [];
  let bytes = 0;
  for await (const chunk of process.stdin) {
    bytes += chunk.length;
    if (bytes > 1_000_000) throw new Error('Input exceeded 1 MB.');
    chunks.push(chunk);
  }
  const envelope = envelopeSchema.parse(JSON.parse(Buffer.concat(chunks).toString('utf8')));
  const model = process.env.KEVIN_MODEL;
  if (!model) throw new Error('Set KEVIN_MODEL to an already installed local model.');
  const port = process.env.KEVIN_OLLAMA_PORT ?? '11434';
  if (!/^\d{1,5}$/.test(port) || Number(port) < 1 || Number(port) > 65535) throw new Error('Invalid loopback port.');
  const response = await fetch(`http://127.0.0.1:${port}/api/chat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    signal: AbortSignal.timeout(25_000),
    body: JSON.stringify({
      model, stream: false,
      messages: [
        { role: 'system', content: envelope.system },
        { role: 'user', content: JSON.stringify(envelope.data) },
      ],
      format: envelope.outputSchema,
      options: { temperature: 0, seed: 42 },
    }),
  });
  if (!response.ok) throw new Error(`Ollama HTTP ${response.status}`);
  const body = await response.json();
  if (typeof body?.message?.content !== 'string') throw new Error('Invalid Ollama response.');
  const output = JSON.parse(body.message.content);
  process.stdout.write(JSON.stringify(output));
} catch (error) {
  process.stderr.write(`${error instanceof Error ? error.message : 'Ollama bridge failed'}\n`);
  process.exitCode = 1;
}
