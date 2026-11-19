// Caminho relativo: app/web/src/lib/verification.ts
// Starts verifications in one shared worker and lets the page follow and cancel them
// (VER-04). The same worker searches a profile's optimal circuit (MT-42) and runs the
// optimizer (MT-46). The worker is created when the page loads (`prewarm`), so a verification needs
// no network afterwards (GER-05). If the worker dies, the runs in progress fail and the next
// one starts a new worker.
import type { VerificationProgress } from '#lib/core.ts';
import type { FromWorker, SearchKind, ToWorker } from '#lib/verify.worker.ts';

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

/** Searches the optimum of `request` (`CompressionSession.optimalRequest`) and resolves
 *  with the answer for `adoptOptimal`; rejects if the worker fails. */
export type SearchOptimal = (request: string) => Promise<string>;

export class Verifier {
	#worker: WorkerLike | null = null;
	#next = 1;
	#runs = new Map<number, Pending>();
	#searches = new Map<number, { finish: (r: string) => void; fail: (e: Error) => void }>();

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
			if ('search' in m) {
				const s = this.#searches.get(m.search);
				this.#searches.delete(m.search);
				if ('error' in m) s?.fail(new Error(m.error));
				else s?.finish(m.result);
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
			const failed = [...this.#runs.values(), ...this.#searches.values()];
			this.#runs.clear();
			this.#searches.clear();
			w.terminate();
			if (this.#worker === w) this.#worker = null;
			loaded();
			for (const run of failed) run.fail(new Error(e.message ?? ''));
		};
		this.#worker = w;
		return w;
	}

	search = (request: string, kind: SearchKind = 'optimal'): Promise<string> => {
		const worker = this.#connect();
		const id = this.#next++;
		const done = new Promise<string>((finish, fail) => this.#searches.set(id, { finish, fail }));
		worker.postMessage(
			kind === 'optimal' ? { search: id, request } : { search: id, request, kind }
		);
		return done;
	};

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

export const searchOptimalInWorker: SearchOptimal = (request) => shared.search(request);

/** Runs the optimizer on `request` (`CompressionSession.optimizerRequest`) and resolves with
 *  the answer for `adoptOptimizer`; rejects if the worker fails (AJU-14). */
export const searchOptimizerInWorker: SearchOptimal = (request) =>
	shared.search(request, 'optimizer');
