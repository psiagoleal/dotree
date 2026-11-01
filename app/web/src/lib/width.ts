// Caminho relativo: app/web/src/lib/width.ts
// Step 2 of the width challenge (GER-03): how W follows from v_max. Every value comes from
// the core (ADR 0001); this only lays them out as formula, substitution and result, and
// shows W through the powers of two around v_max: 2^(W-1) <= v_max < 2^W.
import type { Input, Matrix } from './core';

export interface WidthPath {
	inputs: Input[];
	/** The combined expression, e.g. `AB + CD + 3E + 3F + G`. */
	symbolic: string;
	/** Each input replaced by its maximum, e.g. `7·7 + 3·3 + …`. */
	substituted: string;
	/** Value of each term, or null when there is a single term (it would repeat v_max). */
	termValues: string | null;
	vMax: string;
	vMaxPlusOne: string;
	lower: { exponent: number; value: string };
	upper: { exponent: number; value: string };
}

const power = (exponent: number) => ({ exponent, value: (2n ** BigInt(exponent)).toString() });

export function widthPath(m: Pick<Matrix, 'terms' | 'inputs' | 'vMax' | 'outputWidth'>): WidthPath {
	const maxOf = new Map(m.inputs.map((i) => [i.var, i.max]));
	const coefficient = (c: string) => (c === '1' ? [] : [c]);
	return {
		inputs: m.inputs,
		symbolic: m.terms.map((t) => coefficient(t.coefficient).join('') + t.vars).join(' + '),
		substituted: m.terms
			.map((t) =>
				[...coefficient(t.coefficient), ...[...t.vars].map((v) => maxOf.get(v))].join('·')
			)
			.join(' + '),
		termValues: m.terms.length > 1 ? m.terms.map((t) => t.max).join(' + ') : null,
		vMax: m.vMax,
		vMaxPlusOne: (BigInt(m.vMax) + 1n).toString(),
		lower: power(m.outputWidth - 1),
		upper: power(m.outputWidth)
	};
}
