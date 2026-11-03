#!/usr/bin/env node
import { parseArgs } from 'node:util';
import { cp, mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { z } from 'zod';
import {
  compile, compose, contextInputSchema, defaultConfig, designVoice, identitySchema,
  parseConfig, renderPrompt, requestSchema, voiceSchema,
} from '../../core/src/index.js';
import { CommandGenerator } from './provider.js';
import { loadConfig, readJson, saveConfig } from './storage.js';
import { confirmVoiceSave, interviewVoice } from './voice-wizard.js';

const help = `Kevin 0.1 — One mind. Many voices. Always you.

kevin init [--language pt-BR] [--style minimalist]
kevin doctor | voices
kevin voice default [name]                       # show or set default voice
kevin voice create <name> | voice edit <name>       # voice JSON on stdin
kevin voice create <name> --description "..."     # guided personality interview
  [--answers <file>] [--save]                     # noninteractive proposal or save
kevin identity import <file> | identity export
kevin reply | rewrite | review | explain | summarize | draft
kevin skill install <codex|claude|cursor|generic> [--destination <directory>]

Compose options:
  --request <file>           Full request JSON (intent, messages, facts, context)
  --intent <text>            Intent; otherwise plain stdin supplies intent/draft
  --platform <name>          slack, github, email, chat, docs, unknown
  --task <name>              reply, rewrite, review, pr-description, explain, summarize, draft
  --audience <name>          teammate, manager, external, public, unknown
  --seriousness <level>      ordinary, sensitive, critical, unknown
  --energy <level>           calm, playful, tense, unknown
  --language <tag>           Explicit output language
  --voice <name>             Manual voice; critical-context constraints still apply
  --blend <name>             Repeat to add named trait overlays in order
  --no-humor                Disable humor and sarcasm
  --json                    Emit full compiled envelope without generating
  --prompt                  Emit portable prompt without generating
  --model-command <exe>      Explicit local command reading envelope JSON on stdin
  --model-arg <arg>          Repeat for command arguments; no shell evaluation
  --config <file>            Default: .kevin/config.json in current directory

No model command: emit prompt to stdout, notice to stderr. Nothing sent externally.
Model command: selected executable receives intent, profile and supplied messages;
it may send them to its provider. Use a local model command for offline generation.
All generated text is a draft. Kevin never posts messages or runs review actions.
`;

async function stdin(): Promise<string> {
  if (process.stdin.isTTY) return '';
  const chunks: Buffer[] = [];
  let bytes = 0;
  for await (const chunk of process.stdin) {
    const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(String(chunk));
    bytes += buffer.length;
    if (bytes > 1_000_000) throw new Error('Input exceeded 1 MB.');
    chunks.push(buffer);
  }
  return Buffer.concat(chunks).toString('utf8').trim();
}

function print(value: unknown): void { process.stdout.write(`${JSON.stringify(value, null, 2)}\n`); }

async function main(): Promise<void> {
  const { values, positionals } = parseArgs({
    allowPositionals: true,
    options: {
      help: { type: 'boolean', short: 'h' }, config: { type: 'string' },
      language: { type: 'string' }, style: { type: 'string' }, destination: { type: 'string' },
      description: { type: 'string' }, answers: { type: 'string' }, save: { type: 'boolean' },
      request: { type: 'string' }, intent: { type: 'string' }, platform: { type: 'string' },
      task: { type: 'string' }, audience: { type: 'string' }, seriousness: { type: 'string' },
      energy: { type: 'string' }, voice: { type: 'string' }, blend: { type: 'string', multiple: true },
      'no-humor': { type: 'boolean' }, json: { type: 'boolean' }, prompt: { type: 'boolean' },
      'model-command': { type: 'string' }, 'model-arg': { type: 'string', multiple: true },
    },
  });
  const [command, action, name] = positionals;
  if (values.help || !command) { process.stdout.write(help); return; }
  const configPath = resolve(values.config ?? '.kevin/config.json');
  if (command === 'init') {
    const config = defaultConfig();
    if (values.language) config.identity.primaryLanguage = values.language;
    if (values.style) config.routing.fallback = values.style;
    const validated = parseConfig(config);
    await mkdir(dirname(configPath), { recursive: true, mode: 0o700 });
    await writeFile(configPath, `${JSON.stringify(validated, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
    process.stdout.write(`Created ${configPath}\n`); return;
  }
  if (command === 'skill' && action === 'install') {
    const targets: Record<string, string> = {
      codex: '.agents/skills', claude: '.claude/skills', cursor: '.cursor/skills', generic: 'skills',
    };
    if (!name || !Object.hasOwn(targets, name)) throw new Error('Choose codex, claude, cursor, or generic.');
    const root = resolve(dirname(fileURLToPath(import.meta.url)), '../../../..');
    const destination = resolve(values.destination ?? targets[name] ?? 'skills', 'kevin');
    const source = resolve(root, 'skills/kevin');
    if (destination === source) throw new Error('Skill already lives at this destination.');
    await mkdir(dirname(destination), { recursive: true });
    await cp(source, destination, { recursive: true, errorOnExist: true, force: false });
    process.stdout.write(`Installed ${destination}\n`); return;
  }
  const config = await loadConfig(configPath);
  if (command === 'doctor') {
    print({ version: '0.1.0', node: process.versions.node, config: configPath,
      voices: Object.keys(config.voices).length, generation: 'Agent or explicit --model-command',
      network: 'No built-in network client or telemetry' }); return;
  }
  if (command === 'voices') {
    if (values.json) { print({ default: config.routing.fallback, voices: config.voices }); return; }
    for (const [id, voice] of Object.entries(config.voices)) {
      process.stdout.write(`${id}${id === config.routing.fallback ? ' (default)' : ''}\n  ${voice.description}\n`);
    }
    return;
  }
  if (command === 'voice' && action === 'default') {
    if (!name) { process.stdout.write(`${config.routing.fallback}\n`); return; }
    if (!Object.hasOwn(config.voices, name)) throw new Error(`Unknown voice: ${name}. Run kevin voices to list options.`);
    config.routing.fallback = name;
    await saveConfig(configPath, config);
    process.stdout.write(`Default voice: ${name}. Specific routing rules and --voice can override it.\n`); return;
  }
  if (command === 'identity') {
    if (action === 'export') { print(config.identity); return; }
    if (action === 'import' && name) {
      config.identity = identitySchema.parse(await readJson(resolve(name)));
      await saveConfig(configPath, config); process.stdout.write('Identity imported.\n'); return;
    }
    throw new Error('Use identity import <file> or identity export. Calibration is available in the skill.');
  }
  if (command === 'voice' && ['create', 'edit'].includes(action ?? '') && name) {
    const exists = Object.hasOwn(config.voices, name);
    if (action === 'create' && exists) throw new Error('Voice already exists. Use voice edit.');
    if (action === 'edit' && !exists) throw new Error('Unknown voice. Use voice create.');
    if (values.description !== undefined) {
      if (action !== 'create') throw new Error('Description interviews create new voices. Use JSON stdin for voice edit.');
      const answers = values.answers ? await readJson(resolve(values.answers)) : {};
      // Validate name and description before asking questions or writing anything.
      parseConfig({ ...config, voices: { ...config.voices, [name]: { description: values.description } } });
      const result = process.stdin.isTTY ? await interviewVoice(values.description, answers) : designVoice(values.description, answers);
      if (!result) { process.stdout.write('Cancelled; no voice saved.\n'); return; }
      if (result.status === 'question') {
        if (values.save) throw new Error(`Missing answer: ${result.question.id}. Run without --save to inspect the next question.`);
        print(result); return;
      }
      print(result);
      const save = values.save || (process.stdin.isTTY && await confirmVoiceSave());
      if (!save) { process.stderr.write('Proposal only; no profile changed. Supply --save after reviewing complete answers.\n'); return; }
      const updated = parseConfig({ ...config, voices: { ...config.voices, [name]: result.voice } });
      await saveConfig(configPath, updated);
      process.stderr.write(`Saved voice ${name}. Identity and routing unchanged.\n`); return;
    }
    if (values.answers || values.save) throw new Error('--answers and --save require --description.');
    const candidate = voiceSchema.parse(JSON.parse(await stdin()));
    // Validate the key before assigning it, including special JavaScript property names.
    const updated = parseConfig({ ...config, voices: { ...config.voices, [name]: candidate } });
    await saveConfig(configPath, updated); process.stdout.write(`Saved voice ${name}\n`); return;
  }
  const modes = ['reply', 'rewrite', 'review', 'explain', 'summarize', 'draft'];
  if (!modes.includes(command)) throw new Error(`Unknown command: ${command}. Use --help.`);
  if (action) throw new Error('Unexpected positional text. Use --intent or stdin.');
  const input = values.request ? requestSchema.parse(await readJson(resolve(values.request))) : undefined;
  const body = input ? '' : await stdin();
  const context: Record<string, unknown> = { ...input?.context };
  const contextFlags = ['platform', 'task', 'audience', 'seriousness', 'energy', 'language'] as const;
  for (const key of contextFlags) if (values[key] !== undefined) context[key] = values[key];
  context.task = values.task ?? input?.context.task ?? command;
  const request = requestSchema.parse({
    ...input,
    intent: values.intent ?? input?.intent ?? body,
    ...(!input && command === 'rewrite' ? { draft: body } : {}),
    context: contextInputSchema.parse(context),
    ...(values.voice ? { voice: values.voice } : {}),
    ...(values.blend ? { blend: values.blend } : {}),
    ...(values['no-humor'] ? { humor: 'off' } : {}),
  });
  const envelope = compile(request, config);
  if (values.json) { print(envelope); return; }
  if (values.prompt || !values['model-command']) {
    if (!values.prompt) process.stderr.write('No model selected. Output is a prompt, not a generated reply.\n');
    process.stdout.write(renderPrompt(envelope)); return;
  }
  const result = await compose(envelope, new CommandGenerator(values['model-command'], values['model-arg']));
  if (result.status === 'draft') process.stdout.write(`${result.text}\n`);
}

main().catch((error: unknown) => {
  if (error instanceof z.ZodError) {
    process.stderr.write(`Invalid data: ${error.issues.map(issue => `${issue.path.join('.')}: ${issue.message}`).join('; ')}\n`);
  } else if (error instanceof Error && 'code' in error && error.code === 'EEXIST') {
    process.stderr.write('Destination already exists; no overwrite performed.\n');
  } else {
    process.stderr.write(`${error instanceof Error ? error.message : 'Unknown error'}\n`);
  }
  process.exitCode = 1;
});
