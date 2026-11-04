// Caminho relativo: app/web/src/lib/verify.worker.ts
// Runs a verification off the main thread (VER-04): loads its own copy of the core, checks
// the circuit chunk by chunk and posts the progress after each one. Cancelling terminates
// the worker, so there is no message for it.
import init, { Verification } from '#lib/wasm/dotree.js';
import type { VerificationProgress } from '#lib/core.ts';

export type ToWorker = { netlist: string };
export type FromWorker = { progress: VerificationProgress } | { error: string };

/** Vectors per chunk: a few milliseconds of work, so progress moves smoothly. */
const CHUNK = 1 << 16;

const post = (m: FromWorker) => self.postMessage(m);

self.onmessage = async (e: MessageEvent<ToWorker>) => {
	try {
		await init();
		const v = new Verification(e.data.netlist);
		for (;;) {
			const progress: VerificationProgress = JSON.parse(v.run(CHUNK));
			post({ progress });
			if (progress.done) break;
		}
		v.free();
	} catch (err) {
		post({ error: String(err) });
	}
};
