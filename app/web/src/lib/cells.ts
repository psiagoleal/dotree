// Caminho relativo: app/web/src/lib/cells.ts
// Names of the cell kinds (CMP-20, ADR 0005): FA, HA and the one-column counters 7:3 and
// 15:4. The instance names are those of the core, the same in the diagram, the wiring table
// and the VHDL (spec 05); the sentences come from the messages.
import type { CellKind } from '#lib/core.ts';
import { m } from '#lib/paraglide/messages.js';

/** Short name: FA, HA, 7:3, 15:4. */
export const KIND_SHORT: Record<CellKind, string> = {
	fa: 'FA',
	ha: 'HA',
	c73: '7:3',
	c154: '15:4'
};

/** Points a cell of each kind takes. */
export const KIND_INPUTS: Record<CellKind, number> = { fa: 3, ha: 2, c73: 7, c154: 15 };

export const isCounter = (k: CellKind) => k === 'c73' || k === 'c154';

/** "FA", "HA", "contador 7:3" ("7:3 counter"). */
export function kindText(k: CellKind): string {
	return isCounter(k) ? m.cell_counter({ kind: KIND_SHORT[k] }) : KIND_SHORT[k];
}

/** A concluded cell for the screen and screen readers: "FA 3", "contador 7:3, célula 3". */
export function cellName(c: { kind: CellKind; number: number }): string {
	return isCounter(c.kind)
		? m.cell_counter_concluded({ kind: KIND_SHORT[c.kind], number: c.number })
		: m.cell_concluded({ kind: KIND_SHORT[c.kind], number: c.number });
}

/** The core's instance name: FA3, HA3, C73_3, C154_3. */
export function instanceName(k: CellKind, n: number): string {
	return isCounter(k) ? `${k.toUpperCase()}_${n}` : `${k.toUpperCase()}${n}`;
}
