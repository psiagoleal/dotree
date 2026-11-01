// Caminho relativo: app/web/src/lib/expression.ts
// Input helpers for the expression form. Not domain rules: the core validates everything.

/** Upper-case letters in the text, sorted and without repetition: one width field each. */
export function variablesIn(src: string): string[] {
	return [...new Set(src.match(/[A-Z]/g) ?? [])].sort();
}

/** Width offered for a variable the user has not sized yet. */
export const DEFAULT_WIDTH = 4;

export interface Example {
	id: 'r1' | 'r3' | 'r4';
	expression: string;
	widths: Record<string, number>;
}

/** Built-in examples (EXP-12): the reference cases of spec 00. */
export const EXAMPLES: Example[] = [
	{
		id: 'r1',
		expression: 'A*B + C*D + 3(E+F) + G',
		widths: { A: 3, B: 3, C: 2, D: 2, E: 2, F: 1, G: 1 }
	},
	{ id: 'r3', expression: 'A', widths: { A: 4 } },
	{ id: 'r4', expression: 'A*B', widths: { A: 4, B: 4 } }
];
