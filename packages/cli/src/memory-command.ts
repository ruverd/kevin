import { randomUUID } from 'node:crypto';
import { memoryEntrySchema } from '../../core/src/index.js';
import type { Config } from '../../core/src/index.js';
import { loadMemory, saveMemory } from './storage.js';

interface MemoryOptions {
  voice: string | undefined;
  text: string | undefined;
  kind: string | undefined;
  all: boolean | undefined;
}

export async function memoryCommand(path: string, config: Config, action: string | undefined, id: string | undefined, options: MemoryOptions): Promise<unknown> {
  if (options.voice && !Object.hasOwn(config.voices, options.voice)) throw new Error('Unknown memory voice. Run kevin voices to list options.');
  if (options.voice && options.all) throw new Error('Choose --voice or --all, not both.');
  const memory = await loadMemory(path);
  const items = options.voice ? (Object.hasOwn(memory.voices, options.voice) ? memory.voices[options.voice] ?? [] : []) : memory.global;
  if (action === 'list' || action === undefined) {
    if (options.all) return memory;
    return { scope: options.voice ?? 'global', entries: items };
  }
  if (options.all) throw new Error('--all is available only for memory list.');
  if (action === 'add') {
    const entry = memoryEntrySchema.parse({ id: randomUUID(), kind: options.kind ?? 'note', text: options.text, createdAt: new Date().toISOString(), source: 'user' });
    items.push(entry);
    if (options.voice) memory.voices[options.voice] = items;
    await saveMemory(path, memory);
    return { scope: options.voice ?? 'global', entry };
  }
  if (action === 'remove') {
    if (!id) throw new Error('Use memory remove <id> with the same --voice scope if applicable.');
    const index = items.findIndex(entry => entry.id === id);
    if (index < 0) throw new Error('Memory ID not found in selected scope.');
    items.splice(index, 1);
    if (options.voice && !items.length) delete memory.voices[options.voice];
    await saveMemory(path, memory);
    return { removed: id, scope: options.voice ?? 'global' };
  }
  throw new Error('Use memory add --text <text>, memory list, or memory remove <id>.');
}
