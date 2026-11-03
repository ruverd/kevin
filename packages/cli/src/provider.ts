import { spawn } from 'node:child_process';
import type { Envelope, Generator } from '../../core/src/index.js';

export class CommandGenerator implements Generator {
  constructor(private readonly executable: string, private readonly args: string[] = []) {}

  async generate(envelope: Envelope): Promise<unknown> {
    return new Promise((resolve, reject) => {
      const child = spawn(this.executable, this.args, { shell: false, stdio: ['pipe', 'pipe', 'pipe'] });
      const chunks: Buffer[] = [];
      let bytes = 0;
      let settled = false;
      const finish = (error?: Error, result?: unknown) => {
        if (settled) return;
        settled = true; clearTimeout(timer);
        if (error) { child.kill('SIGKILL'); reject(error); } else resolve(result);
      };
      const timer = setTimeout(() => finish(new Error('Model command exceeded 30 seconds.')), 30_000);
      child.on('error', () => finish(new Error('Cannot start model command. Check executable path.')));
      child.stdin.on('error', () => finish(new Error('Model command closed stdin before reading request.')));
      child.stdout.on('data', (chunk: Buffer) => {
        bytes += chunk.length;
        if (bytes > 1_000_000) finish(new Error('Model output exceeded 1 MB.'));
        else chunks.push(chunk);
      });
      // Drain stderr without printing provider logs, which may contain private input or credentials.
      child.stderr.resume();
      child.on('close', (code) => {
        if (settled) return;
        if (code !== 0) { finish(new Error(`Model command failed (exit ${code ?? 'signal'}).`)); return; }
        try { finish(undefined, JSON.parse(Buffer.concat(chunks).toString('utf8'))); }
        catch { finish(new Error('Model command must return one JSON object; no fences or logging on stdout.')); }
      });
      child.stdin.end(JSON.stringify(envelope));
    });
  }
}
