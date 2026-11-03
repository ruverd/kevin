import type { Config, Context, Request, Traits, Voice } from './schema.js';
import type { Analysis } from './context.js';

export interface ResolvedVoice {
  traits: Voice['traits'];
  descriptions: string[];
  behaviors: string[];
  avoid: string[];
}

function mergeTraits(base: Traits, overlay: Voice['traits']): Traits {
  return {
    directness: overlay.directness ?? base.directness,
    formality: overlay.formality ?? base.formality,
    warmth: overlay.warmth ?? base.warmth,
    humor: overlay.humor ?? base.humor,
    sarcasm: overlay.sarcasm ?? base.sarcasm,
    verbosity: overlay.verbosity ?? base.verbosity,
  };
}

export function resolveVoice(config: Config, name: string): ResolvedVoice {
  const voice: Voice | undefined = Object.hasOwn(config.voices, name) ? config.voices[name] : undefined;
  if (!voice) throw new Error(`Unknown voice: ${name}`);
  const parent = voice.extends ? resolveVoice(config, voice.extends) : { traits: {}, descriptions: [], behaviors: [], avoid: [] };
  return {
    traits: { ...parent.traits, ...voice.traits },
    descriptions: [...parent.descriptions, voice.description],
    behaviors: [...parent.behaviors, ...voice.behaviors], avoid: [...parent.avoid, ...voice.avoid],
  };
}

export interface Route {
  voices: string[];
  descriptions: string[];
  traits: Traits;
  behaviors: string[];
  avoid: string[];
  preferences: string[];
  reasons: string[];
}

export function routeVoice(request: Request, config: Config, analysis: Analysis): Route {
  const { context } = analysis;
  const rule = config.routing.rules
    .map((value, index) => ({ value, index }))
    .sort((a, b) => b.value.priority - a.value.priority || a.index - b.index)
    .find(({ value }) => Object.entries(value.when).every(([key, value]) =>
      Object.entries(context).some(([field, actual]) => field === key && actual === value)))?.value;
  const automatic = analysis.confidence === 'low' ? config.routing.fallback : rule?.voice ?? config.routing.fallback;
  const name = request.voice ?? automatic;
  const reasons = [...analysis.reasons, request.voice ? `Manual voice: ${name}` : `Auto voice: ${name}`];
  if (!request.voice && rule && analysis.confidence !== 'low') reasons.push(`Rule: ${rule.name} (${rule.priority})`);
  const language = Object.hasOwn(config.identity.languages, context.language) ? config.identity.languages[context.language] : undefined;
  const route: Route = {
    voices: [name, ...request.blend],
    traits: mergeTraits(config.identity.traits, language?.traits ?? {}),
    descriptions: [], behaviors: [], avoid: [...config.identity.avoid],
    preferences: [...config.identity.preferences, ...language?.preferences ?? []], reasons,
  };
  for (const voice of route.voices) {
    const resolved = resolveVoice(config, voice);
    route.traits = mergeTraits(route.traits, resolved.traits);
    route.descriptions.push(...resolved.descriptions);
    route.behaviors.push(...resolved.behaviors); route.avoid.push(...resolved.avoid);
  }
  if (!request.voice && analysis.confidence !== 'low' && rule) route.traits = mergeTraits(route.traits, rule.overrides);
  if (context.formality !== undefined) route.traits.formality = context.formality;
  if (request.blend.length) reasons.push('Blending applies named traits in order; last explicit trait wins.');
  return route;
}

export interface HumorDecision {
  mode: 'off' | 'optional' | 'invited';
  styles: string[];
  reason: string;
}

export function constrainHumor(request: Request, context: Context, route: Route): HumorDecision {
  let reason: string | undefined;
  if (request.humor === 'off') reason = 'User disabled humor.';
  else if (context.seriousness !== 'ordinary') reason = 'Seriousness is sensitive, critical, or unknown.';
  else if (context.energy === 'tense') reason = 'Tense context.';
  else if (['review', 'pr-description'].includes(context.task)) reason = 'Technical findings and evidence take priority.';
  else if (['external', 'public', 'unknown'].includes(context.audience)) reason = 'No established audience permission for humor.';
  else if (!context.existingHumor && context.energy !== 'playful') reason = 'No conversational invitation for humor.';
  else if (route.traits.humor === 0) reason = 'Selected identity or voice disables humor.';
  if (reason) {
    route.traits.humor = 0; route.traits.sarcasm = 0;
    return { mode: 'off', styles: [], reason };
  }
  // Sarcasm needs an actual humorous exchange, not merely a playful energy label.
  if (!context.existingHumor) route.traits.sarcasm = 0;
  return {
    mode: context.existingHumor ? 'invited' : 'optional',
    styles: ['understatement', 'dry observation', 'playful exaggeration'],
    reason: 'Humor may fit; omit it whenever forced or less clear than a plain reply.',
  };
}
