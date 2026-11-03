import { createInterface } from 'node:readline/promises';
import { designVoice } from '../../core/src/index.js';
import type { DesignResult } from '../../core/src/index.js';

export async function interviewVoice(description: string, initialAnswers: unknown): Promise<Extract<DesignResult, { status: 'proposal' }> | undefined> {
  const reader = createInterface({ input: process.stdin, output: process.stdout });
  try {
    let result = designVoice(description, initialAnswers);
    while (result.status === 'question') {
      const { question } = result;
      process.stdout.write(`\n${question.prompt}\n${question.why}\n`);
      if (question.evidence) process.stdout.write(`${question.evidence}\n`);
      for (const [index, option] of question.options.entries()) {
        process.stdout.write(`${index + 1}. ${option.label}${option.id === question.recommended ? ' (recommended)' : ''}\n`);
      }
      const answer = (await reader.question('Choose number or option ID; Enter accepts recommendation; q cancels: ')).trim();
      if (answer.toLowerCase() === 'q') return undefined;
      const option = answer === '' ? question.options.find(option => option.id === question.recommended)
        : question.options.find((option, index) => option.id === answer || String(index + 1) === answer);
      if (!option) { process.stdout.write('Choose one listed option.\n'); continue; }
      result = designVoice(description, { ...result.answers, [question.id]: option.id });
    }
    return result;
  } finally { reader.close(); }
}

export async function confirmVoiceSave(): Promise<boolean> {
  const reader = createInterface({ input: process.stdin, output: process.stdout });
  try {
    const answer = (await reader.question('Save this voice to the selected profile? [y/N] ')).trim().toLowerCase();
    return answer === 'y' || answer === 'yes';
  } finally { reader.close(); }
}
