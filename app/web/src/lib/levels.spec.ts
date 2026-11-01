// Caminho relativo: app/web/src/lib/levels.spec.ts
import { describe, expect, it } from 'vitest';
import { columnLayout, formatHalves, visibleTop } from './levels';
import type { LevelPoint, Snapshot } from './core';

const pt = (id: number, column: number, signal = `p${id}`): LevelPoint => ({
	id,
	column,
	ready: 0,
	signal,
	origin: 'input',
	term: 0
});

describe('columnLayout (CMP-05)', () => {
	it('puts each cell as a group of its inputs, in input order, then the loose points', () => {
		const points = [pt(0, 1), pt(1, 1), pt(2, 1), pt(3, 1), pt(4, 0)];
		const groups = [
			{ kind: 'ha' as const, inputs: [3, 1] },
			{ kind: 'fa' as const, inputs: [9, 9, 9], column: 5 }
		];
		const col1 = columnLayout(points, groups, 1);
		expect(col1.groups).toEqual([{ index: 0, kind: 'ha', points: [points[3], points[1]] }]);
		expect(col1.loose.map((p) => p.id)).toEqual([0, 2]);
		expect(columnLayout(points, groups, 0).loose.map((p) => p.id)).toEqual([4]);
		expect(columnLayout(points, groups, 0).groups).toEqual([]);
	});

	it('boxes a CPA cell around the points it takes from the level; the carry-in is not one', () => {
		// FA in column 1 of a CPA: two points of the level plus the carry (id 7) from column 0.
		const points = [pt(0, 1), pt(1, 1), pt(2, 0)];
		const layout = columnLayout(points, [{ kind: 'fa', inputs: [0, 1, 7] }], 1);
		expect(layout.groups).toEqual([{ index: 0, kind: 'fa', points: [points[0], points[1]] }]);
		expect(layout.loose).toEqual([]);
	});
});

describe('visibleTop (EXP-09)', () => {
	const snap = {
		levels: [{ points: [pt(0, 2)], cells: [], cpa: false }],
		current: { points: [pt(1, 4), pt(2, 0)], pending: [] }
	} as unknown as Snapshot;

	it('is the highest column with a point in any panel while W is hidden', () => {
		expect(visibleTop(snap, null)).toBe(4);
	});

	it('reaches W - 1 once W is revealed', () => {
		expect(visibleTop(snap, 7)).toBe(6);
		expect(visibleTop(snap, 3)).toBe(4);
	});
});

describe('formatHalves', () => {
	it('writes half units in the locale', () => {
		expect(formatHalves(0, 'pt-br')).toBe('0');
		expect(formatHalves(5, 'pt-br')).toBe('2,5');
		expect(formatHalves(5, 'en')).toBe('2.5');
		expect(formatHalves(12, 'en')).toBe('6');
	});
});
