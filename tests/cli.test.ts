import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { compile, defaultConfig } from '../packages/core/src/index.js';
import { CommandGenerator } from '../packages/cli/src/provider.js';

const cli = fileURLToPath(new URL('../packages/cli/src/main.js', import.meta.url));

function run(cwd: string, args: string[], input = '') {
  return spawnSync(process.execPath, [cli, ...args], { cwd, input, encoding: 'utf8', timeout: 10_000 });
}

test('CLI init, routing, doctor, no-model fallback, profile/voice edits, skill install and stdio generation', async () => {
  const cwd = await mkdtemp(resolve(tmpdir(), 'kevin-cli-'));
  try {
    assert.notEqual(run(cwd, ['doctor']).status, 0);
    assert.equal(run(cwd, ['init', '--language', 'pt-BR']).status, 0);
    assert.notEqual(run(cwd, ['init']).status, 0);
    const doctor = run(cwd, ['doctor']);
    assert.equal(doctor.status, 0, doctor.stderr);
    assert.equal(JSON.parse(doctor.stdout).voices, 7);
    const exportProfile = run(cwd, ['identity', 'export']);
    await writeFile(resolve(cwd, 'identity.json'), exportProfile.stdout);
    assert.equal(run(cwd, ['identity', 'import', 'identity.json']).status, 0);
    const create = run(cwd, ['voice', 'create', 'custom'], JSON.stringify({ description: 'Custom', extends: 'minimalist', traits: { warmth: 0.8 } }));
    assert.equal(create.status, 0, create.stderr);
    assert.notEqual(run(cwd, ['voice', 'create', '../escape'], '{"description":"bad"}').status, 0);
    assert.equal(run(cwd, ['voice', 'edit', 'custom'], '{"description":"Changed"}').status, 0);
    const input = {
      intent: 'Acknowledge the coffee joke.',
      context: { platform: 'slack', audience: 'teammate', seriousness: 'ordinary', energy: 'playful', existingHumor: true },
      messages: ['Next you will automate my coffee.'],
    };
    await writeFile(resolve(cwd, 'request.json'), JSON.stringify(input));
    const result = run(cwd, ['reply', '--request', 'request.json', '--json']);
    assert.equal(result.status, 0, result.stderr);
    const compiled = JSON.parse(result.stdout);
    assert.equal(compiled.data.plan.route.voices[0], 'witty-engineer');
    assert.equal(compiled.data.plan.context.language, 'pt-BR');
    const prompt = run(cwd, ['reply'], 'Hello.');
    assert.equal(prompt.status, 0, prompt.stderr);
    assert.match(prompt.stderr, /prompt, not a generated reply/);
    assert.match(prompt.stdout, /INPUT DATA/);
    assert.notEqual(run(cwd, ['reply', '--platform', 'invalid'], 'Hi').status, 0);
    assert.notEqual(run(cwd, ['reply', '--voice', 'missing'], 'Hi').status, 0);
    const generation = run(cwd, ['reply', '--request', 'request.json', '--model-command', process.execPath,
      '--model-arg=-e', '--model-arg=process.stdin.resume();process.stdin.on("end",()=>process.stdout.write(JSON.stringify({status:"draft",text:"Coffee still needs a human in the loop."})))']);
    assert.equal(generation.status, 0, generation.stderr);
    assert.equal(generation.stdout.trim(), 'Coffee still needs a human in the loop.');
    const install = run(cwd, ['skill', 'install', 'codex']);
    assert.equal(install.status, 0, install.stderr);
    const installed = await readFile(resolve(cwd, '.agents/skills/kevin/SKILL.md'), 'utf8');
    assert.match(installed, /name: kevin/);
    assert.match(await readFile(resolve(cwd, '.agents/skills/kevin/references/identity.md'), 'utf8'), /Calibration workflow/);
    assert.notEqual(run(cwd, ['skill', 'install', 'codex']).status, 0);
  } finally { await rm(cwd, { recursive: true, force: true }); }
});

test('provider command rejects invalid JSON, failure, oversized output and missing executable', async () => {
  const envelope = compile({ intent: 'Reply.' }, defaultConfig());
  for (const script of ['process.stdout.write("not-json")', 'process.exit(2)', 'process.stdout.write("x".repeat(1000001))']) {
    await assert.rejects(new CommandGenerator(process.execPath, ['-e', script]).generate(envelope));
  }
  await assert.rejects(new CommandGenerator('/no-such-kevin-executable').generate(envelope), /Cannot start/);
});

test('description-driven voice creation previews first and persists only with explicit save', async () => {
  const cwd = await mkdtemp(resolve(tmpdir(), 'kevin-design-'));
  try {
    assert.equal(run(cwd, ['init']).status, 0);
    const profilePath = resolve(cwd, '.kevin/config.json');
    const original = await readFile(profilePath, 'utf8');
    const initial = run(cwd, ['voice', 'create', 'field-engineer', '--description', 'Dry, concise engineer.']);
    assert.equal(initial.status, 0, initial.stderr);
    assert.equal(JSON.parse(initial.stdout).question.id, 'context');
    const args = ['voice', 'create', 'field-engineer', '--description', 'Dry, concise engineer.', '--answers', 'answers.json'];
    await writeFile(resolve(cwd, 'answers.json'), JSON.stringify({ context: 'slack', stance: 'direct', humor: 'dry', detail: 'short', example: 'a' }));
    const preview = run(cwd, args);
    assert.equal(preview.status, 0, preview.stderr);
    assert.equal(JSON.parse(preview.stdout).status, 'proposal');
    assert.equal(await readFile(profilePath, 'utf8'), original);
    const saved = run(cwd, [...args, '--save']);
    assert.equal(saved.status, 0, saved.stderr);
    const updated = JSON.parse(await readFile(profilePath, 'utf8'));
    assert.equal(updated.voices['field-engineer'].description, 'Dry, concise engineer.');
    assert.deepEqual(updated.identity, JSON.parse(original).identity);
    assert.deepEqual(updated.routing, JSON.parse(original).routing);
    assert.notEqual(run(cwd, [...args, '--save']).status, 0);
    const partial = run(cwd, ['voice', 'create', 'incomplete', '--description', 'Example.', '--save']);
    assert.notEqual(partial.status, 0);
  } finally { await rm(cwd, { recursive: true, force: true }); }
});

test('list voices, change default, and reject invalid defaults without changing the profile', async () => {
  const cwd = await mkdtemp(resolve(tmpdir(), 'kevin-default-'));
  try {
    assert.equal(run(cwd, ['init']).status, 0);
    assert.equal(run(cwd, ['voice', 'default']).stdout.trim(), 'neutral');
    assert.match(run(cwd, ['voices']).stdout, /neutral \(default\)/);
    assert.equal(run(cwd, ['voice', 'default', 'minimalist']).status, 0);
    assert.equal(run(cwd, ['voice', 'default']).stdout.trim(), 'minimalist');
    const listed = JSON.parse(run(cwd, ['voices', '--json']).stdout);
    assert.equal(listed.default, 'minimalist');
    assert.ok(listed.voices['dry-comedian']);
    const auto = JSON.parse(run(cwd, ['reply', '--json'], 'Explain.').stdout);
    assert.deepEqual(auto.data.plan.route.voices, ['minimalist']);
    const before = await readFile(resolve(cwd, '.kevin/config.json'), 'utf8');
    assert.notEqual(run(cwd, ['voice', 'default', 'missing']).status, 0);
    assert.equal(await readFile(resolve(cwd, '.kevin/config.json'), 'utf8'), before);
    const override = JSON.parse(run(cwd, ['reply', '--json', '--voice', 'friendly-teammate'], 'Explain.').stdout);
    assert.deepEqual(override.data.plan.route.voices, ['friendly-teammate']);
  } finally { await rm(cwd, { recursive: true, force: true }); }
});

test('model arguments remain literal and private stderr is not exposed', async () => {
  const envelope = compile({ intent: 'Reply.' }, defaultConfig());
  const literal = '$(touch /tmp/kevin-shell-injection); `echo private`';
  const result = await new CommandGenerator(process.execPath, ['-e',
    'process.stdin.resume();process.stdin.on("end",()=>process.stdout.write(JSON.stringify({status:"draft",text:process.argv[1]})))', literal]).generate(envelope);
  assert.deepEqual(result, { status: 'draft', text: literal });
  await assert.rejects(new CommandGenerator(process.execPath, ['-e', 'process.stderr.write("PRIVATE-TOKEN");process.exit(3)']).generate(envelope), error => error instanceof Error && !error.message.includes('PRIVATE-TOKEN'));
});
