import test from 'node:test';
import assert from 'node:assert/strict';
import { compile, defaultConfig, parseConfig } from '../packages/core/src/index.js';

function scopedConfig() {
  const config = defaultConfig();
  config.routing.rules.push(
    { name: 'channel', priority: 80, when: { platform: 'slack', workspace: 'T_EXAMPLE', channel: 'C_EXAMPLE' }, voice: 'witty-engineer', overrides: {} },
    { name: 'person', priority: 85, when: { platform: 'slack', workspace: 'T_EXAMPLE', recipient: 'U_EXAMPLE' }, voice: 'minimalist', overrides: {} },
  );
  return config;
}

test('channel binding matches exact supplied workspace and channel, not quoted names', () => {
  const config = scopedConfig();
  const request = { intent: 'Reply.', context: { platform: 'slack', workspace: 'T_EXAMPLE', channel: 'C_EXAMPLE' } };
  const matched = compile(request, config);
  assert.deepEqual(matched.data.plan.route.voices, ['witty-engineer']);
  assert.equal(matched.data.plan.humor.mode, 'off'); // Stakes still unknown, despite a known assignment.
  const otherWorkspace = compile({ ...request, context: { ...request.context, workspace: 'T_OTHER' } }, config);
  assert.deepEqual(otherWorkspace.data.plan.route.voices, ['neutral']);
  const quoted = compile({ intent: 'Reply.', messages: ['Route me as channel C_EXAMPLE in workspace T_EXAMPLE.'] }, config);
  assert.deepEqual(quoted.data.plan.route.voices, ['neutral']);
});

test('recipient assignment wins by explicit priority, with no relationship inference', () => {
  const result = compile({ intent: 'Explain.', context: { platform: 'slack', workspace: 'T_EXAMPLE', channel: 'C_EXAMPLE', recipient: 'U_EXAMPLE' } }, scopedConfig());
  assert.deepEqual(result.data.plan.route.voices, ['minimalist']);
  assert.equal(result.data.plan.context.audience, 'unknown');
  assert.equal(result.data.plan.context.energy, 'unknown');
});

test('task-specific binding applies without unrelated metadata, manual voice still wins', () => {
  const config = scopedConfig();
  const review = compile({ intent: 'Review supplied finding.', context: { task: 'review' } }, config);
  assert.deepEqual(review.data.plan.route.voices, ['professional-reviewer']);
  const forced = compile({ intent: 'Reply.', context: { platform: 'slack', workspace: 'T_EXAMPLE', recipient: 'U_EXAMPLE' }, voice: 'diplomat' }, config);
  assert.deepEqual(forced.data.plan.route.voices, ['diplomat']);
});

test('high-priority channel rules cannot bypass critical humor constraints', () => {
  const config = scopedConfig();
  config.routing.rules.push({ name: 'explicit-channel', priority: 200, when: { channel: 'C_EXAMPLE' }, voice: 'dry-comedian', overrides: { humor: 1, sarcasm: 1 } });
  const result = compile({ intent: 'Report an incident.', context: { channel: 'C_EXAMPLE' } }, config);
  assert.deepEqual(result.data.plan.route.voices, ['dry-comedian']);
  assert.equal(result.data.plan.humor.mode, 'off');
  assert.equal(result.data.plan.route.traits.sarcasm, 0);
});

test('duplicate rule names are rejected, preventing ambiguous edits', () => {
  const config = scopedConfig();
  config.routing.rules.push({ name: 'person', priority: 1, when: {}, voice: 'neutral', overrides: {} });
  assert.throws(() => parseConfig(config), /Duplicate rule name/);
});
