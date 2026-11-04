// Caminho relativo: app/web/src/lib/verification.ts
// Starts a verification in a worker and lets the page follow and cancel it (VER-04).
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
	/** The final progress, or null when cancelled; rejects if the worker fails. */
	done: Promise<VerificationProgress | null>;
	cancel(): void;
}

export type StartVerification = (
	netlist: string,
	onprogress: (p: VerificationProgress) => void
) => VerificationRun;

function defaultWorker(): WorkerLike {
	return new Worker(new URL('./verify.worker.ts', import.meta.url), { type: 'module' });
}

export function startVerification(
	netlist: string,
	onprogress: (p: VerificationProgress) => void,
	createWorker: () => WorkerLike = defaultWorker
): VerificationRun {
	const worker = createWorker();
	let settled = false;
	let finish!: (p: VerificationProgress | null) => void;
	let fail!: (e: Error) => void;
	const done = new Promise<VerificationProgress | null>((resolve, reject) => {
		finish = resolve;
		fail = reject;
	});
	// Every way out stops the worker, and nothing it sends afterwards counts.
	const end = (then: () => void) => {
		if (settled) return;
		settled = true;
		worker.terminate();
		then();
	};
	worker.onmessage = (e) => {
		if (settled) return;
		const m = e.data;
		if ('error' in m) {
			end(() => fail(new Error(m.error)));
			return;
		}
		onprogress(m.progress);
		if (m.progress.done) end(() => finish(m.progress));
	};
	worker.onerror = (e) => end(() => fail(new Error(e.message)));
	worker.postMessage({ netlist });
	return { done, cancel: () => end(() => finish(null)) };
}
