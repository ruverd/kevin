import { z } from 'zod';

const voiceId = z.string().regex(/^[a-z][a-z0-9-]{0,63}$/);
export const memoryEntrySchema = z.strictObject({
  id: z.string().min(1).max(100),
  kind: z.enum(['preference', 'note']),
  text: z.string().trim().min(1).max(2000),
  createdAt: z.iso.datetime(),
  source: z.literal('user'),
});
export type MemoryEntry = z.infer<typeof memoryEntrySchema>;
const entries = z.array(memoryEntrySchema).max(100);
export const memorySchema = z.strictObject({
  version: z.literal(1),
  global: entries.default([]),
  voices: z.record(voiceId, entries).default({}),
}).superRefine((memory, ctx) => {
  for (const [scope, items] of [['global', memory.global], ...Object.entries(memory.voices)] satisfies [string, MemoryEntry[]][]) {
    const ids = new Set<string>();
    for (const entry of items) {
      if (ids.has(entry.id)) ctx.addIssue({ code: 'custom', message: `Duplicate memory ID in ${scope}` });
      ids.add(entry.id);
    }
  }
});
export type Memory = z.infer<typeof memorySchema>;

export function emptyMemory(): Memory { return memorySchema.parse({ version: 1 }); }

export function selectMemory(memoryInput: unknown, selectedVoices: string[]) {
  const memory = memorySchema.parse(memoryInput);
  return {
    global: memory.global,
    voices: [...new Set(selectedVoices)].flatMap(voice => {
      const items = Object.hasOwn(memory.voices, voice) ? memory.voices[voice] : undefined;
      return items?.length ? [{ voice, entries: items }] : [];
    }),
  };
}
