import { z } from 'zod';

const unit = z.number().min(0).max(1);
const text = z.string().max(20_000);
export const traitsSchema = z.strictObject({
  directness: unit, formality: unit, warmth: unit, humor: unit,
  sarcasm: unit, verbosity: unit,
});
export type Traits = z.infer<typeof traitsSchema>;
export const baseTraits: Traits = {
  directness: 0.75, formality: 0.35, warmth: 0.5, humor: 0.25, sarcasm: 0, verbosity: 0.3,
};
const id = z.string().regex(/^[a-z][a-z0-9-]{0,63}$/);
export const voiceSchema = z.strictObject({
  description: text,
  extends: id.optional(),
  traits: traitsSchema.partial().default({}),
  behaviors: z.array(text).max(30).default([]),
  avoid: z.array(text).max(30).default([]),
});
export type Voice = z.infer<typeof voiceSchema>;
export const contextSchema = z.strictObject({
  platform: z.enum(['slack', 'github', 'email', 'chat', 'docs', 'unknown']).default('unknown'),
  task: z.enum(['reply', 'rewrite', 'review', 'pr-description', 'explain', 'summarize', 'draft']).default('reply'),
  audience: z.enum(['teammate', 'manager', 'external', 'public', 'unknown']).default('unknown'),
  seriousness: z.enum(['ordinary', 'sensitive', 'critical', 'unknown']).default('unknown'),
  energy: z.enum(['calm', 'playful', 'tense', 'unknown']).default('unknown'),
  existingHumor: z.boolean().default(false),
  responseNeeded: z.boolean().default(true),
  language: z.string().min(2).max(35).default('en-US'),
  formality: unit.optional(),
  workspace: z.string().min(1).max(200).optional(),
  channel: z.string().min(1).max(200).optional(),
  recipient: z.string().min(1).max(200).optional(),
});
export type Context = z.infer<typeof contextSchema>;
// Removing defaults matters: partial(defaultedSchema) still inserts defaults in Zod 4.
// A rule must match only fields it declares; omitted request fields must not overwrite profile defaults.
export const contextInputSchema = z.strictObject({
  platform: contextSchema.shape.platform.removeDefault().optional(),
  task: contextSchema.shape.task.removeDefault().optional(),
  audience: contextSchema.shape.audience.removeDefault().optional(),
  seriousness: contextSchema.shape.seriousness.removeDefault().optional(),
  energy: contextSchema.shape.energy.removeDefault().optional(),
  existingHumor: contextSchema.shape.existingHumor.removeDefault().optional(),
  responseNeeded: contextSchema.shape.responseNeeded.removeDefault().optional(),
  language: contextSchema.shape.language.removeDefault().optional(),
  formality: unit.optional(),
  workspace: contextSchema.shape.workspace,
  channel: contextSchema.shape.channel,
  recipient: contextSchema.shape.recipient,
});
const matchSchema = contextInputSchema.omit({ formality: true });
export const ruleSchema = z.strictObject({
  name: z.string().min(1).max(100), priority: z.number().int().default(0),
  when: matchSchema, voice: id, overrides: traitsSchema.partial().default({}),
});
export const identitySchema = z.strictObject({
  name: z.string().max(100).default(''),
  primaryLanguage: z.string().min(2).max(35).default('en-US'),
  traits: traitsSchema.default(baseTraits),
  preferences: z.array(text).max(100).default([]),
  avoid: z.array(text).max(100).default([]),
  languages: z.record(z.string(), z.strictObject({
    traits: traitsSchema.partial().default({}), preferences: z.array(text).default([]),
  })).default({}),
});
export const configSchema = z.strictObject({
  version: z.literal(1),
  identity: identitySchema.default(identitySchema.parse({})),
  voices: z.record(id, voiceSchema),
  routing: z.strictObject({
    fallback: id.default('neutral'), rules: z.array(ruleSchema).max(100).default([]),
  }).default({ fallback: 'neutral', rules: [] }),
}).superRefine((config, ctx) => {
  const missing = (name: string, path: PropertyKey[]) => {
    if (!Object.hasOwn(config.voices, name)) ctx.addIssue({ code: 'custom', path, message: `Unknown voice: ${name}` });
  };
  missing(config.routing.fallback, ['routing', 'fallback']);
  config.routing.rules.forEach((rule, i) => missing(rule.voice, ['routing', 'rules', i, 'voice']));
  const ruleNames = new Set<string>();
  config.routing.rules.forEach((rule, index) => {
    if (ruleNames.has(rule.name)) ctx.addIssue({ code: 'custom', path: ['routing', 'rules', index, 'name'], message: 'Duplicate rule name' });
    ruleNames.add(rule.name);
  });
  for (const [name, voice] of Object.entries(config.voices)) {
    if (voice.extends) missing(voice.extends, ['voices', name, 'extends']);
    const seen = new Set<string>([name]);
    let parent = voice.extends;
    while (parent && Object.hasOwn(config.voices, parent)) {
      if (seen.has(parent)) {
        ctx.addIssue({ code: 'custom', path: ['voices', name], message: 'Voice inheritance cycle' }); break;
      }
      seen.add(parent); parent = config.voices[parent]?.extends;
    }
  }
});
export type Config = z.infer<typeof configSchema>;

export const requestSchema = z.strictObject({
  intent: z.string().min(1).max(20_000),
  context: contextInputSchema.default({}),
  messages: z.array(text).max(100).default([]),
  draft: text.optional(),
  facts: z.array(text).max(100).default([]),
  preserve: z.array(z.string().min(1).max(2000)).max(100).default([]),
  voice: id.optional(),
  blend: z.array(id).max(5).default([]),
  humor: z.enum(['auto', 'off']).default('auto'),
});
export type Request = z.infer<typeof requestSchema>;
export const outputSchema = z.discriminatedUnion('status', [
  z.strictObject({ status: z.literal('draft'), text: z.string().trim().min(1).max(40_000) }),
  z.strictObject({ status: z.literal('no-response'), text: z.literal('') }),
]);
export type Output = z.infer<typeof outputSchema>;

export function parseConfig(value: unknown): Config {
  // Future formats must get explicit migrations; do not guess at renamed fields.
  return configSchema.parse(value);
}
