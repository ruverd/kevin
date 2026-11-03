import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { parseArgs } from 'node:util';
import { compile, compose, defaultConfig } from '../packages/core/src/index.js';
import { CommandGenerator } from '../packages/cli/src/provider.js';
import { scenarios } from './scenarios.js';

const { values } = parseArgs({ options: {
  'model-command': { type: 'string' }, 'model-arg': { type: 'string', multiple: true },
  'model-id': { type: 'string' }, 'settings': { type: 'string' },
  'output-dir': { type: 'string' },
} });
if (values['model-command'] && (!values['model-id'] || !values.settings || !values['output-dir'])) {
  throw new Error('Model runs require --model-id, --settings and --output-dir for reproducibility. Settings are a recorded label; configure the actual generator separately.');
}
let assertions = 0;
const records: unknown[] = [];
for (const scenario of scenarios) {
  for (const forced of [false, true]) {
    const request = { ...scenario.request, ...(forced ? { voice: 'dry-comedian' } : {}) };
    const envelope = compile(request, defaultConfig());
    assert.equal(envelope.data.plan.route.voices[0], forced ? 'dry-comedian' : scenario.expectedVoice, scenario.id);
    assert.equal(envelope.data.plan.humor.mode, scenario.expectedHumor, scenario.id);
    if (scenario.expectedHumor === 'off') {
      assert.equal(envelope.data.plan.route.traits.humor, 0);
      assert.equal(envelope.data.plan.route.traits.sarcasm, 0);
    }
    assertions += 1;
    if (values['model-command']) {
      const started = Date.now();
      try {
        const output = await compose(envelope, new CommandGenerator(values['model-command'], values['model-arg']));
        records.push({ id: scenario.id, forced, envelope, output, elapsedMs: Date.now() - started, semanticReview: 'pending' });
      } catch (error) {
        records.push({ id: scenario.id, forced, envelope, error: error instanceof Error ? error.message : 'Unknown failure', semanticReview: 'pending' });
        process.exitCode = 1;
      }
    }
  }
}
if (values['model-command'] && values['output-dir']) {
  await mkdir(values['output-dir'], { recursive: true });
  const path = `${values['output-dir']}/run-${Date.now()}.json`;
  await writeFile(path, JSON.stringify({
    version: 1, date: new Date().toISOString(), model: values['model-id'], settings: values.settings,
    command: values['model-command'], args: values['model-arg'] ?? [], records,
  }, null, 2), { flag: 'wx', mode: 0o600 });
  process.stdout.write(`Recorded model outputs at ${path}. Subjective quality requires review.\n`);
}
process.stdout.write(`${assertions} routing scenarios passed (auto + forced). No subjective quality score inferred.\n`);
