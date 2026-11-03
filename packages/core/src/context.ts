import { contextSchema } from './schema.js';
import type { Config, Context, Request } from './schema.js';

// Detection only tightens constraints. Quoted content never selects voices or changes preferences.
const critical = /\b(incident|outage|sev[ -]?[01]|data loss|security|vulnerability|credential leak|breach|incidente|indisponibilidade|perda de dados|vulnerabilidade|seguran[çc]a)\b/iu;
const sensitive = /\b(blocked|approval|disagreement|harassment|layoff|fired|stressed|bloquead[oa]|aprova[çc][aã]o|desentendimento|ass[eé]dio|demiss[aã]o|estressad[oa])\b/iu;

export interface Analysis {
  context: Context;
  confidence: 'low' | 'supplied';
  reasons: string[];
}

export function analyzeContext(request: Request, config: Config): Analysis {
  const context = contextSchema.parse({ language: config.identity.primaryLanguage, ...request.context });
  const reasons: string[] = [];
  const content = [request.intent, request.draft ?? '', ...request.facts, ...request.messages].join('\n');
  if (critical.test(content)) {
    context.seriousness = 'critical'; reasons.push('Conservative critical-topic detection.');
  } else if (sensitive.test(content) && context.seriousness !== 'critical') {
    context.seriousness = 'sensitive'; reasons.push('Conservative sensitive-topic detection.');
  }
  const confidence = context.seriousness === 'unknown' || context.platform === 'unknown' ? 'low' : 'supplied';
  if (confidence === 'low') reasons.push('Incomplete context: use matching configured rules or the default; unknown stakes disable humor.');
  return { context, confidence, reasons };
}
