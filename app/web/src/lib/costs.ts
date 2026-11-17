// Caminho relativo: app/web/src/lib/costs.ts
// GER-12 (MT-45): the cost fields of the settings panel. The core keeps costs in whole
// hundredths of A_FA and T_FA; the panel shows and reads them as decimals, with a comma or
// a point, so "0,3" and "0.3" both mean 30.
import { COST_MAX, COST_MIN, DEFAULT_COSTS, type Costs } from '#lib/core.ts';
import { formatHundredths } from '#lib/levels.ts';

/** The four fields every exercise has, in the panel's order. */
export const COST_KEYS = ['faArea', 'faDelay', 'haArea', 'haDelay'] as const;
export type CostKey = (typeof COST_KEYS)[number];
/** The counters' fields (ADR 0005): blank means the cost of the counter's FA network. */
export const COUNTER_KEYS = ['c73Area', 'c73Delay', 'c154Area', 'c154Delay'] as const;
export type CounterKey = (typeof COUNTER_KEYS)[number];

/** What a counter costs without a cost of its own: 4 FA, 3 deep; 11 FA, 5 deep. */
export function networkCost(c: Costs, k: CounterKey): number {
	const times = { c73Area: 4, c73Delay: 3, c154Area: 11, c154Delay: 5 }[k];
	return times * (k.endsWith('Area') ? c.faArea : c.faDelay);
}

/** The cost in force for a counter field: its own, or its network's. */
export function counterCost(c: Costs, k: CounterKey): number {
	return c[k] ?? networkCost(c, k);
}

/** A decimal typed by the user, in whole hundredths; null when it is not a number with at
 *  most two decimals from 0.01 to 100. */
export function parseCost(text: string): number | null {
	const t = text.trim().replace(',', '.');
	if (!/^\d+(\.\d{0,2})?$|^\.\d{1,2}$/.test(t)) return null;
	const n = Math.round(Number(t) * 100);
	return n >= COST_MIN && n <= COST_MAX ? n : null;
}

/** The text of a field: 30 -> "0,3" in Portuguese, "0.3" in English. */
export function costText(n: number, locale: string): string {
	return formatHundredths(n, locale).replace(/\s/g, '');
}

export function isDefaultCosts(c: Costs): boolean {
	return (
		COST_KEYS.every((k) => c[k] === DEFAULT_COSTS[k]) &&
		COUNTER_KEYS.every((k) => c[k] === undefined)
	);
}
