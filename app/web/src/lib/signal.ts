// Caminho relativo: app/web/src/lib/signal.ts
// Splits a signal identifier into the letter-and-index pairs drawn in the indexed view
// (VIS-01): `a1_b0` -> a₁b₀. The identifiers themselves come from the core.

export interface SignalPart {
	name: string;
	index: string;
}

const PART = /^([A-Za-z]+)(\d+)$/;

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
