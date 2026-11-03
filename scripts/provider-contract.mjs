import { z } from 'zod';
export const envelopeSchema = z.object({
  version: z.literal(1), system: z.string(), data: z.record(z.string(), z.unknown()),
  outputSchema: z.record(z.string(), z.unknown()),
});
