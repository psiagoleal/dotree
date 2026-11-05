// Caminho relativo: app/web/src/lib/verification.ts
// Starts verifications in one shared worker and lets the page follow and cancel them
// (VER-04). The worker is created when the page loads (`prewarm`), so a verification needs
// no network afterwards (GER-05). If the worker dies, the runs in progress fail and the next
// one starts a new worker.
import type { VerificationProgress } from '#lib/core.ts';
import type { FromWorker, ToWorker } from '#lib/verify.worker.ts';

/** The part of `Worker` used here, so tests can pass a fake. */
export interface WorkerLike {
	postMessage(message: ToWorker): void;
	terminate(): void;
	onmessage: ((e: MessageEvent<FromWorker>) => void) | null;
	onerror: ((e: ErrorEvent) => void) | null;
}

export interface VerificationRun {
	/** The final progress, or null when cancelled; rejects if the worker fails. An empty
	 *  error message means the worker itself could not load. */
	done: Promise<VerificationProgress | null>;
	cancel(): void;
}

export type StartVerification = (
	netlist: string,
	onprogress: (p: VerificationProgress) => void
) => VerificationRun;

interface Pending {
	onprogress: (p: VerificationProgress) => void;
	finish: (p: VerificationProgress | null) => void;
	fail: (e: Error) => void;
}

function defaultWorker(): WorkerLike {
	return new Worker(new URL('./verify.worker.ts', import.meta.url), { type: 'module' });
}

export class Verifier {
	#worker: WorkerLike | null = null;
	#next = 1;
	#runs = new Map<number, Pending>();

	readonly #create: () => WorkerLike;

	constructor(createWorker: () => WorkerLike = defaultWorker) {
		this.#create = createWorker;
	}

	/** Starts the worker (and its copy of the core) ahead of the first run; resolves once it
	 *  has loaded, or failed to, so the page can wait for it as part of loading. */
	prewarm(): Promise<void> {
		this.#connect();
		return this.#ready;
	}

	#ready: Promise<void> = Promise.resolve();

	#connect(): WorkerLike {
		if (this.#worker) return this.#worker;
		const w = this.#create();
		let loaded!: () => void;
		this.#ready = new Promise((resolve) => (loaded = resolve));
		w.onmessage = (e) => {
			const m = e.data;
			if ('ready' in m) {
				loaded();
				return;
			}
			const run = this.#runs.get(m.run);
			if (!run) return; // cancelled or finished: nothing it sends counts
			if ('error' in m) {
				this.#runs.delete(m.run);
				run.fail(new Error(m.error));
				return;
			}
			run.onprogress(m.progress);
			if (m.progress.done) {
				this.#runs.delete(m.run);
				run.finish(m.progress);
			}
		};
		w.onerror = (e) => {
			const failed = [...this.#runs.values()];
			this.#runs.clear();
			w.terminate();
			if (this.#worker === w) this.#worker = null;
			loaded();
			for (const run of failed) run.fail(new Error(e.message ?? ''));
		};
		this.#worker = w;
		return w;
	}

	start: StartVerification = (netlist, onprogress) => {
		const worker = this.#connect();
		const id = this.#next++;
		const done = new Promise<VerificationProgress | null>((finish, fail) => {
			this.#runs.set(id, { onprogress, finish, fail });
		});
		worker.postMessage({ run: id, netlist });
		return {
			done,
			cancel: () => {
				const run = this.#runs.get(id);
				if (!run) return;
				this.#runs.delete(id);
				this.#worker?.postMessage({ cancel: id });
				run.finish(null);
			}
		};
	};
}

const shared = new Verifier();

export const startVerification: StartVerification = (netlist, onprogress) =>
	shared.start(netlist, onprogress);

export const prewarmVerification = (): Promise<void> => shared.prewarm();
