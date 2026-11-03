import { z } from 'zod';
import { voiceSchema } from './schema.js';
import type { Voice } from './schema.js';

export const designAnswersSchema = z.strictObject({
  context: z.enum(['slack', 'review', 'email', 'mixed']).optional(),
  stance: z.enum(['direct', 'warm', 'tactful']).optional(),
  humor: z.enum(['none', 'dry', 'playful']).optional(),
  detail: z.enum(['short', 'balanced', 'deep']).optional(),
  example: z.enum(['a', 'b', 'c']).optional(),
  avoid: z.array(z.string().min(1).max(1000)).max(20).default([]),
});
export type DesignAnswers = z.infer<typeof designAnswersSchema>;
type QuestionId = 'context' | 'stance' | 'humor' | 'detail' | 'example';
export interface DesignQuestion {
  id: QuestionId;
  prompt: string;
  why: string;
  recommended: string;
  options: { id: string; label: string }[];
  evidence?: string;
}
export interface DesignPreview {
  kind: 'illustrative-fixture';
  context: string;
  evidence: string;
  text: string;
}
export type DesignResult =
  | { status: 'question'; description: string; answers: DesignAnswers; question: DesignQuestion }
  | { status: 'proposal'; description: string; answers: DesignAnswers; voice: Voice; previews: DesignPreview[] };

const questions: DesignQuestion[] = [
  {
    id: 'context', prompt: 'Where should this voice be most useful?',
    why: 'Audience and task shape the voice. This does not install automatic routing rules.',
    recommended: 'mixed', options: [
      { id: 'slack', label: 'Ordinary Slack conversations with teammates' },
      { id: 'review', label: 'Technical reviews with evidence and concrete next steps' },
      { id: 'email', label: 'External email with careful wording' },
      { id: 'mixed', label: 'Mixed contexts, with restrained defaults' },
    ],
  },
  {
    id: 'stance', prompt: 'When directness and warmth compete, what should lead?',
    why: 'This selects a starting archetype without changing the base identity.',
    recommended: 'direct', options: [
      { id: 'direct', label: 'Lead with the useful point, retain necessary warmth' },
      { id: 'warm', label: 'Sound collaborative and approachable, without generic praise' },
      { id: 'tactful', label: 'Use careful attribution and tact, without hiding disagreement' },
    ],
  },
  {
    id: 'humor', prompt: 'When an ordinary teammate exchange invites humor, what fits?',
    why: 'Humor stays optional and is disabled for serious, blocked or uncertain contexts.',
    recommended: 'none', options: [
      { id: 'none', label: 'No humor or sarcasm' },
      { id: 'dry', label: 'A small dry observation or understatement, never contempt' },
      { id: 'playful', label: 'Light situational exaggeration, no jokes aimed at people' },
    ],
  },
  {
    id: 'detail', prompt: 'How much detail should this voice prefer?',
    why: 'Shorter delivery must still preserve evidence, qualifications and commitments.',
    recommended: 'balanced', options: [
      { id: 'short', label: 'Short: essential facts and next action' },
      { id: 'balanced', label: 'Balanced: answer plus the reason needed to act' },
      { id: 'deep', label: 'Detailed: explain reasoning and meaningful tradeoffs' },
    ],
  },
];

function exampleQuestion(context: DesignAnswers['context']): DesignQuestion {
  if (context === 'review') return {
    id: 'example', prompt: 'Which review comment sounds closest to this voice?',
    why: 'Choose expression, not a different technical claim.', recommended: 'a',
    evidence: 'Synthetic finding: items[0].id throws when items is empty. No test result is supplied.',
    options: [
      { id: 'a', label: 'items[0].id throws when items is empty. Please guard the empty case before accessing it.' },
      { id: 'b', label: 'Can we guard the empty case here? items[0].id throws when items is empty.' },
      { id: 'c', label: 'Empty input causes items[0].id to throw. An explicit guard is needed before that access.' },
    ],
  };
  if (context === 'email') return {
    id: 'example', prompt: 'Which email wording sounds closest to this voice?',
    why: 'All options keep the same request and add no deadline.', recommended: 'c',
    evidence: 'Synthetic intent: ask an external stakeholder to send the missing document. No deadline supplied.',
    options: [
      { id: 'a', label: 'Please send the missing document.' },
      { id: 'b', label: 'Could you send the missing document? Thanks.' },
      { id: 'c', label: 'Could you please send the missing document?' },
    ],
  };
  return {
    id: 'example', prompt: 'Which teammate reply sounds closest to this voice?',
    why: 'Every choice keeps the commitment to look, without promising a fix.', recommended: 'a',
    evidence: 'Synthetic intent: say you will look after lunch. Tests have not run. Do not promise a fix.',
    options: [
      { id: 'a', label: "I'll take a look after lunch. Tests haven't run yet." },
      { id: 'b', label: "Thanks for flagging it. I'll take a look after lunch. Tests haven't run yet." },
      { id: 'c', label: 'I will review it after lunch. Tests have not run yet.' },
    ],
  };
}

export function designVoice(descriptionInput: unknown, answersInput: unknown = {}): DesignResult {
  const description = z.string().trim().min(1).max(20_000).parse(descriptionInput);
  const answers = designAnswersSchema.parse(answersInput);
  const question = questions.find(question => answers[question.id] === undefined);
  if (question) return { status: 'question', description, answers, question: structuredClone(question) };
  const example = exampleQuestion(answers.context);
  if (!answers.example) return { status: 'question', description, answers, question: example };
  const parent = answers.stance === 'warm' ? 'friendly-teammate' : answers.stance === 'tactful' ? 'diplomat' : 'minimalist';
  const selected = example.options.find(option => option.id === answers.example);
  if (!selected) throw new Error('Unknown example selection.');
  const voice = voiceSchema.parse({
    description,
    extends: parent,
    traits: {
      directness: answers.stance === 'direct' ? 0.9 : answers.stance === 'warm' ? 0.75 : 0.65,
      warmth: answers.stance === 'warm' ? 0.75 : 0.5,
      formality: answers.context === 'email' || answers.example === 'c' ? 0.7 : 0.3,
      humor: answers.humor === 'none' ? 0 : answers.humor === 'dry' ? 0.35 : 0.5,
      sarcasm: 0,
      verbosity: answers.detail === 'short' ? 0.15 : answers.detail === 'deep' ? 0.75 : 0.4,
    },
    behaviors: [
      `Preferred starting context: ${answers.context}. Adapt to the actual audience and task.`,
      answers.example === 'b' ? 'Use a brief collaborative acknowledgement when useful.'
        : answers.example === 'c' ? 'Prefer complete sentences and a measured register.' : 'Lead with the useful point.',
      ...(answers.humor === 'dry' ? ['When invited, prefer understated situational observations.'] : []),
      ...(answers.humor === 'playful' ? ['When invited, light situational exaggeration is acceptable.'] : []),
    ],
    avoid: ['Invented facts or commitments', 'Forced humor', ...answers.avoid],
  });
  return {
    status: 'proposal', description, answers, voice,
    previews: [
      { kind: 'illustrative-fixture', context: `${answers.context ?? 'mixed'} example`, evidence: example.evidence ?? '', text: selected.label },
      { kind: 'illustrative-fixture', context: 'Critical incident; humor disabled regardless of voice',
        evidence: 'Synthetic incident: requests are failing; cause is unknown; no ETA supplied.',
        text: 'Requests are failing. The cause is still unknown.' },
    ],
  };
}
