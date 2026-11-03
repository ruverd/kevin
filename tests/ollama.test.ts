import test from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { once } from 'node:events';
import { fileURLToPath } from 'node:url';
import { compile, compose, defaultConfig } from '../packages/core/src/index.js';
import { CommandGenerator } from '../packages/cli/src/provider.js';

test('Ollama bridge sends separate roles and schema to a real loopback protocol fixture', async () => {
  let received: unknown;
  let route: string | undefined;
  const server = createServer(async (req, res) => {
    route = req.url;
    const chunks: Buffer[] = [];
    for await (const chunk of req) chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(String(chunk)));
    received = JSON.parse(Buffer.concat(chunks).toString());
    res.setHeader('Content-Type', 'application/json');
    res.end(JSON.stringify({ message: { content: JSON.stringify({ status: 'draft', text: 'Guard items[0].id first.' }) } }));
  });
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  try {
    const address = server.address();
    assert.ok(address && typeof address !== 'string');
    const bridge = fileURLToPath(new URL('../../scripts/ollama.mjs', import.meta.url));
    const envelope = compile({ intent: 'Write a review.', preserve: ['items[0].id'] }, defaultConfig());
    const generator = new CommandGenerator(process.execPath, ['-e',
      'process.env.KEVIN_MODEL="fixture";process.env.KEVIN_OLLAMA_PORT=process.argv[1];import(process.argv[2]);',
      String(address.port), bridge]);
    const result = await compose(envelope, generator);
    assert.equal(result.text, 'Guard items[0].id first.');
    assert.equal(route, '/api/chat');
    assert.deepEqual(received, {
      model: 'fixture', stream: false,
      messages: [{ role: 'system', content: envelope.system }, { role: 'user', content: JSON.stringify(envelope.data) }],
      format: envelope.outputSchema, options: { temperature: 0, seed: 42 },
    });
  } finally {
    server.closeAllConnections(); server.close(); await once(server, 'close');
  }
});
