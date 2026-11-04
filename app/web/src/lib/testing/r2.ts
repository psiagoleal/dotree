// Caminho relativo: app/web/src/lib/testing/r2.ts
// Test fixture: the R2 tree of spec 00 driven straight on a session (no interface).
import type { CompressionSession, Core } from '#lib/core.ts';
import { EXAMPLES } from '#lib/expression.ts';

export const R1 = EXAMPLES.find((e) => e.id === 'r1')!;

/** Forms a cell by signal; throws if the core refuses it. */
export function formOn(s: CompressionSession, column: number, signals: string[]) {
	const points = s.snapshot().current.points;
	const ids = signals.map((sig) => {
		const p = points.find((q) => q.column === column && q.signal === sig);
		if (!p) throw new Error(`no ${sig} in column ${column}`);
		return p.id;
	});
	if (!s.formCell(ids).ok) throw new Error(`refused: ${signals.join(', ')}`);
}

const TREE: [number, string[]][][] = [
	[
		[1, ['a0_b1', 'a1_b0', 'c0_d1']],
		[1, ['c1_d0', 'e1', 'e0']],
		[2, ['a0_b2', 'a1_b1']]
	],
	[
		[1, ['f0', 's1', 's2']],
		[2, ['a2_b0', 'c1_d1', 'e1']],
		[2, ['s3', 'k1', 'k2']],
		[3, ['a1_b2', 'a2_b1', 'k3']]
	],
	[
		[0, ['a0_b0', 'c0_d0', 'e0']],
		[2, ['s5', 'k4', 's6']],
		[3, ['k5', 's7', 'k6']],
		[4, ['a2_b2', 'k7']]
	]
];

/** R1 with the first `levels` levels of the R2 tree concluded (3: ready for the CPA). */
export function r2Levels(core: Core, levels: number): CompressionSession {
	const s = core.openSession(R1.expression, R1.widths);
	for (const cells of TREE.slice(0, levels)) {
		for (const [column, signals] of cells) formOn(s, column, signals);
		if (!s.concludeLevel().ok) throw new Error('level not concluded');
	}
	return s;
}

/** R2 complete: the tree and the CPA. */
export function r2(core: Core): CompressionSession {
	const s = r2Levels(core, 3);
	if (!s.closeWithCpa().ok) throw new Error('CPA refused');
	return s;
}
