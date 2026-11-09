// Caminho relativo: app/web/src/lib/signal.ts
// Splits a signal identifier into the letter-and-index pairs drawn in the indexed view
// (VIS-01): `a1_b0` -> a₁b₀. The identifiers themselves come from the core.

export interface SignalPart {
	name: string;
	index: string;
}

const PART = /^([A-Za-z]+)(\d+)$/;

/** Identifier of the fixed dots of the additive constant (EXP-15), drawn as `1`. */
export const FIXED_ONE = 'one';

/** What a dot is called on screen and to assistive technology: `1` for a fixed dot. */
export const signalLabel = (signal: string) => (signal === FIXED_ONE ? '1' : signal);

/** Letter and index of each factor, or null when the identifier has another shape. */
export function signalParts(signal: string): SignalPart[] | null {
	const parts: SignalPart[] = [];
	for (const piece of signal.split('_')) {
		const match = PART.exec(piece);
		if (!match) return null;
		parts.push({ name: match[1], index: match[2] });
	}
	return parts;
}
