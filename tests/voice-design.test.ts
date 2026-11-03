import test from 'node:test';
import assert from 'node:assert/strict';
import { compile, defaultConfig, designVoice } from '../packages/core/src/index.js';

const answers = { context: 'slack', stance: 'direct', humor: 'dry', detail: 'short', example: 'a', avoid: ['Generic praise'] };

test('interview asks one gap at a time with meaningful recommendations', () => {
  const first = designVoice('Confident, concise, dry humor.');
  assert.equal(first.status, 'question');
  if (first.status !== 'question') throw new Error('Expected question');
  assert.equal(first.question.id, 'context');
  assert.ok(first.question.options.some(option => option.id === first.question.recommended));
  const second = designVoice(first.description, { context: 'review' });
  assert.equal(second.status, 'question');
  if (second.status === 'question') assert.equal(second.question.id, 'stance');
});

test('example choices change with primary context and retain supplied synthetic facts', () => {
  for (const context of ['slack', 'review', 'email']) {
    const result = designVoice('Technical colleague.', { ...answers, context, example: undefined });
    assert.equal(result.status, 'question');
    if (result.status !== 'question') throw new Error('Expected example question');
    assert.equal(result.question.id, 'example');
    assert.ok(result.question.evidence?.startsWith('Synthetic'));
    if (context === 'review') for (const option of result.question.options) assert.match(option.label, /items\[0\]\.id/);
  }
});

test('complete choices produce a voice proposal, previews and no identity mutation', () => {
  const config = defaultConfig();
  const before = structuredClone(config);
  const result = designVoice('Confident, concise, dry humor, no arrogance.', answers);
  assert.equal(result.status, 'proposal');
  if (result.status !== 'proposal') throw new Error('Expected proposal');
  assert.equal(result.voice.extends, 'minimalist');
  assert.equal(result.voice.traits.humor, 0.35);
  assert.equal(result.previews.length, 2);
  assert.ok(result.previews.every(preview => preview.kind === 'illustrative-fixture'));
  assert.deepEqual(config, before);
  config.voices.custom = result.voice;
  const compiled = compile({ intent: 'Report an incident.', voice: 'custom' }, config);
  assert.equal(compiled.data.plan.humor.mode, 'off');
  assert.ok(compiled.data.plan.route.descriptions.includes(result.voice.description));
});

test('invalid choices and description fail before a voice exists', () => {
  assert.throws(() => designVoice('', answers));
  assert.throws(() => designVoice('A voice.', { ...answers, humor: 'always' }));
  assert.throws(() => designVoice('A voice.', { ...answers, autoSend: true }));
});
