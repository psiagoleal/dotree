// Caminho relativo: app/web/src/lib/verify.worker.ts
// Runs verifications off the main thread (VER-04). One worker serves the whole visit: it
// loads its copy of the core as soon as it starts, so nothing is fetched after the page
// has loaded (GER-05). Each run has an id; between chunks the worker yields, so a cancel
// message for that id stops it within one chunk.
import init, { Verification } from '#lib/wasm/dotree.js';
import type { VerificationProgress } from '#lib/core.ts';

export type ToWorker = { run: number; netlist: string } | { cancel: number };
export type FromWorker =
	| { ready: true }
	| { run: number; progress: VerificationProgress }
	| { run: number; error: string };

/** Vectors per chunk: a few milliseconds of work, so progress moves smoothly. */
const CHUNK = 1 << 16;

// Loading starts with the worker; it says when it is ready, so the page can count it as
// part of loading (GER-05). A failed load is tried again by the next run.
let ready = init();
ready.then(
	() => self.postMessage({ ready: true } satisfies FromWorker),
	() => self.postMessage({ ready: true } satisfies FromWorker)
);
const cancelled = new Set<number>();
const post = (m: FromWorker) => self.postMessage(m);

// Yields to pending messages without setTimeout's minimum delay.
const channel = new MessageChannel();
const pause = () =>
	new Promise<void>((resolve) => {
		channel.port1.onmessage = () => resolve();
		channel.port2.postMessage(0);
	});

async function verify(run: number, netlist: string) {
	try {
		await ready.catch(() => (ready = init()));
		await ready;
		const v = new Verification(netlist);
		try {
			while (!cancelled.has(run)) {
				const progress: VerificationProgress = JSON.parse(v.run(CHUNK));
				post({ run, progress });
				if (progress.done) break;
				await pause();
			}
		} finally {
			v.free();
		}
	} catch (err) {
		post({ run, error: String(err) });
	} finally {
		cancelled.delete(run);
	}
}

self.onmessage = (e: MessageEvent<ToWorker>) => {
	const m = e.data;
	if ('cancel' in m) cancelled.add(m.cancel);
	else void verify(m.run, m.netlist);
};
