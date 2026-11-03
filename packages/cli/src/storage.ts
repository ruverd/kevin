import { mkdir, readFile, rename, writeFile, rm } from 'node:fs/promises';
import { dirname } from 'node:path';
import { randomUUID } from 'node:crypto';
import { emptyMemory, memorySchema, parseConfig } from '../../core/src/index.js';
import type { Config, Memory } from '../../core/src/index.js';

export async function readJson(path: string): Promise<unknown> {
  return JSON.parse(await readFile(path, 'utf8'));
}

export async function loadConfig(path: string): Promise<Config> {
  try { return parseConfig(await readJson(path)); }
  catch (error) {
    if (error instanceof Error && 'code' in error && error.code === 'ENOENT') {
      throw new Error(`No profile at ${path}. Run kevin init or pass --config.`);
    }
    throw error;
  }
}

export async function saveConfig(path: string, config: Config): Promise<void> {
  await saveJson(path, parseConfig(config));
}

export async function loadMemory(path: string): Promise<Memory> {
  try { return memorySchema.parse(await readJson(path)); }
  catch (error) {
    if (error instanceof Error && 'code' in error && error.code === 'ENOENT') return emptyMemory();
    throw error;
  }
}

export async function saveMemory(path: string, memory: Memory): Promise<void> {
  await saveJson(path, memorySchema.parse(memory));
}

async function saveJson(path: string, value: unknown): Promise<void> {
  await mkdir(dirname(path), { recursive: true, mode: 0o700 });
  const temporary = `${path}.${randomUUID()}.tmp`;
  try {
    await writeFile(temporary, `${JSON.stringify(value, null, 2)}\n`, { mode: 0o600, flag: 'wx' });
    await rename(temporary, path);
  } finally { await rm(temporary, { force: true }); }
}
