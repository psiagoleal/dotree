// Caminho relativo: app/web/src/lib/verification.spec.ts
// The shared verification worker (VER-04, GER-05): runs by id, cancel by message, one
// worker for the whole visit, a new one after a failure.
import { describe, expect, it } from 'vitest';
import type { VerificationProgress } from '#lib/core.ts';
import type { FromWorker, ToWorker } from '#lib/verify.worker.ts';
import { Verifier, type WorkerLike } from './verification.ts';

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

function setup() {
	const workers: FakeWorker[] = [];
	const verifier = new Verifier(() => {
		const w = new FakeWorker();
		workers.push(w);
		return w;
	});
	const seen: number[] = [];
	const start = () => verifier.start('{"net":1}', (p) => seen.push(p.checked));
	return { verifier, workers, seen, start };
}

describe('Verifier', () => {
	it('prewarm starts the worker before any run and resolves once it has loaded', async () => {
		const { verifier, workers, start } = setup();
		let loaded = false;
		const warm = verifier.prewarm().then(() => (loaded = true));
		expect(workers).toHaveLength(1);
		await Promise.resolve();
		expect(loaded).toBe(false);
		workers[0].emit({ ready: true });
		await warm;
		expect(loaded).toBe(true);
		start();
		start();
		expect(workers).toHaveLength(1);
		expect(workers[0].sent).toEqual([
			{ run: 1, netlist: '{"net":1}' },
			{ run: 2, netlist: '{"net":1}' }
		]);
	});

	it('forwards progress until done and keeps the worker alive', async () => {
		const { workers, seen, start } = setup();
		const run = start();
		workers[0].emit({ run: 1, progress: progress(40) });
		workers[0].emit({ run: 1, progress: progress(100, true) });
		expect(await run.done).toEqual(progress(100, true));
		expect(seen).toEqual([40, 100]);
		expect(workers[0].terminated).toBe(false);
	});

	it('cancelling sends a message, ends with null and drops later messages', async () => {
		const { workers, seen, start } = setup();
		const run = start();
		workers[0].emit({ run: 1, progress: progress(40) });
		run.cancel();
		expect(workers[0].sent.at(-1)).toEqual({ cancel: 1 });
		workers[0].emit({ run: 1, progress: progress(100, true) });
		expect(await run.done).toBeNull();
		expect(seen).toEqual([40]);
		expect(workers[0].terminated).toBe(false);
	});

	it('messages of one run never reach another', async () => {
		const { workers, start } = setup();
		const first = start();
		first.cancel();
		const second = start();
		workers[0].emit({ run: 1, progress: progress(100, true) });
		workers[0].emit({ run: 2, progress: progress(100, true) });
		expect(await first.done).toBeNull();
		expect(await second.done).toEqual(progress(100, true));
	});

	it('an error in a run rejects only that run', async () => {
		const { workers, start } = setup();
		const run = start();
		workers[0].emit({ run: 1, error: 'boom' });
		await expect(run.done).rejects.toThrow('boom');
		expect(workers[0].terminated).toBe(false);
	});

	it('a worker that fails rejects its runs and is replaced on the next run', async () => {
		const { workers, start } = setup();
		const run = start();
		workers[0].onerror?.({ message: 'no module' } as ErrorEvent);
		await expect(run.done).rejects.toThrow('no module');
		expect(workers[0].terminated).toBe(true);
		start();
		expect(workers).toHaveLength(2);
	});
});
