// Caminho relativo: app/web/src/lib/verification.spec.ts
import { describe, expect, it } from 'vitest';
import type { VerificationProgress } from '#lib/core.ts';
import type { FromWorker, ToWorker } from '#lib/verify.worker.ts';
import { startVerification, type WorkerLike } from './verification.ts';

class FakeWorker implements WorkerLike {
	sent: ToWorker[] = [];
	terminated = false;
	onmessage: ((e: MessageEvent<FromWorker>) => void) | null = null;
	onerror: ((e: ErrorEvent) => void) | null = null;
	postMessage(m: ToWorker) {
		this.sent.push(m);
	}
	terminate() {
		this.terminated = true;
	}
	emit(data: FromWorker) {
		this.onmessage?.({ data } as MessageEvent<FromWorker>);
	}
}

const progress = (checked: number, done = false): VerificationProgress => ({
	checked,
	total: 100,
	coverage: 'exhaustive',
	done,
	counterexample: null
});

function start() {
	const worker = new FakeWorker();
	const seen: number[] = [];
	const run = startVerification(
		'{"net":1}',
		(p) => seen.push(p.checked),
		() => worker
	);
	return { worker, seen, run };
}

describe('startVerification', () => {
	it('sends the netlist and forwards progress until done', async () => {
		const { worker, seen, run } = start();
		expect(worker.sent).toEqual([{ netlist: '{"net":1}' }]);
		worker.emit({ progress: progress(40) });
		worker.emit({ progress: progress(100, true) });
		expect(await run.done).toEqual(progress(100, true));
		expect(seen).toEqual([40, 100]);
		expect(worker.terminated).toBe(true);
	});

	it('cancelling stops the worker, ends with null and drops later messages', async () => {
		const { worker, seen, run } = start();
		worker.emit({ progress: progress(40) });
		run.cancel();
		expect(worker.terminated).toBe(true);
		worker.emit({ progress: progress(100, true) });
		expect(await run.done).toBeNull();
		expect(seen).toEqual([40]);
	});

	it('a worker error rejects and stops the worker', async () => {
		const { worker, run } = start();
		worker.emit({ error: 'boom' });
		await expect(run.done).rejects.toThrow('boom');
		expect(worker.terminated).toBe(true);
	});

	it('a worker that fails to load rejects too', async () => {
		const { worker, run } = start();
		worker.onerror?.({ message: 'no module' } as ErrorEvent);
		await expect(run.done).rejects.toThrow('no module');
		expect(worker.terminated).toBe(true);
	});
});
