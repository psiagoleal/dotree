// Caminho relativo: app/web/src/lib/levels.ts
// Layout helpers for the level panels (CMP-05). They only arrange what the core returned.
import type { CellKind, LevelPoint, Snapshot } from './core';

/** A cell drawn over a panel: concluded or pending. */
export interface Group {
	kind: CellKind;
	inputs: number[];
}

export interface ColumnLayout {
	/** Cells of this column, each with its inputs in input order; `index` is into `groups`. */
	groups: { index: number; kind: CellKind; points: LevelPoint[] }[];
	/** Points of this column in no cell, in canonical order. */
	loose: LevelPoint[];
}

/** One column of a panel: the cells first, as boxes around their inputs, then the rest. */
export function columnLayout(points: LevelPoint[], groups: Group[], column: number): ColumnLayout {
	const here = points.filter((p) => p.column === column);
	const byId = new Map(here.map((p) => [p.id, p]));
	const used = new Set<number>();
	const out: ColumnLayout['groups'] = [];
	groups.forEach((g, index) => {
		const ps = g.inputs.map((id) => byId.get(id));
		if (ps.length === 0 || ps.some((p) => p === undefined)) return;
		for (const p of ps) used.add(p!.id);
		out.push({ index, kind: g.kind, points: ps as LevelPoint[] });
	});
	return { groups: out, loose: here.filter((p) => !used.has(p.id)) };
}

/**
 * Highest column drawn in every panel, so the panels line up by weight. While W is hidden it
 * is the highest column holding a point (EXP-09); once revealed, at least W - 1.
 */
export function visibleTop(s: Pick<Snapshot, 'levels' | 'current'>, width: number | null): number {
	const columns = [...s.levels.flatMap((l) => l.points), ...s.current.points].map((p) => p.column);
	return Math.max(0, (width ?? 0) - 1, ...columns);
}

/** A time or area in half units (FA = 2, HA = 1), written in the locale: 5 -> "2,5". */
export function formatHalves(n: number, locale: string): string {
	return new Intl.NumberFormat(locale === 'pt-br' ? 'pt-BR' : 'en', {
		maximumFractionDigits: 1
	}).format(n / 2);
}
