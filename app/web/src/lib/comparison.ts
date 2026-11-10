// Caminho relativo: app/web/src/lib/comparison.ts
// MT-38: the figures of a solution for the side-by-side comparison, read from its snapshot,
// and the frontier of the non-dominated ones in area and delay. Areas and delays are in
// A_FA and T_FA here (the core gives halves).
import type { Snapshot } from '#lib/core.ts';

export interface Figures {
	complete: boolean;
	area: number;
	delay: number;
	fa: number;
	ha: number;
	andGates: number;
	/** Concluded levels before the CPA. */
	levels: number;
	/** Cells of the CPA, one per column it adds; null before the CPA. */
	cpaWidth: number | null;
	/** Cells on some path of maximum delay (CMP-15). */
	critical: number;
	areaDelay: number;
	areaDelay2: number;
}

export function figures(s: Snapshot): Figures {
	const area = s.metrics.area / 2;
	const delay = s.metrics.delay / 2;
	const cpa = s.levels.find((l) => l.cpa);
	return {
		complete: s.result !== null,
		area,
		delay,
		fa: s.metrics.fa,
		ha: s.metrics.ha,
		andGates: s.metrics.andGates,
		levels: s.levels.filter((l) => !l.cpa).length,
		cpaWidth: cpa ? cpa.cells.length : null,
		critical: s.critical.cells.length + s.critical.pending.length,
		areaDelay: area * delay,
		areaDelay2: area * delay * delay
	};
}

/** For each candidate, whether no other complete one has area and delay both at most its
 *  own with at least one smaller; null for an incomplete circuit, outside the frontier. */
export function nonDominated(
	list: { area: number; delay: number; complete: boolean }[]
): (boolean | null)[] {
	return list.map((a) =>
		a.complete
			? !list.some(
					(b) =>
						b.complete &&
						b.area <= a.area &&
						b.delay <= a.delay &&
						(b.area < a.area || b.delay < a.delay)
				)
			: null
	);
}
