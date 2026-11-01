// Caminho relativo: app/web/src/lib/core.ts
// Loads the dotree-core WebAssembly module and types what it returns. The interface never
// reimplements domain rules (ADR 0001); it only calls into this module.
import init, { buildMatrix as wasmBuildMatrix, coreVersion } from '#lib/wasm/dotree.js';

export interface Term {
	/** Decimal string: coefficients can exceed a JS number. */
	coefficient: string;
	vars: string;
	/** Value of the term with every input at its maximum (decimal string). */
	max: string;
}

/** A variable in use, with its width and largest value `2^width - 1`. */
export interface Input {
	var: string;
	width: number;
	max: string;
}

export interface Factor {
	var: string;
	bit: number;
}

export interface Dot {
	id: number;
	column: number;
	/** Index into `Matrix.terms`, which picks the color (COR-01). */
	term: number;
	/** Identifier shared by every artifact: `a1_b0`, `e0`. */
	signal: string;
	factors: Factor[];
}

export interface Matrix {
	terms: Term[];
	inputs: Input[];
	dots: Dot[];
	heights: number[];
	outputWidth: number;
	vMax: string;
	andGates: number;
	warnings: CoreWarning[];
}

export type CoreWarning = { code: 'unusedWidth'; var: string };

export type CoreError =
	| {
			code: 'syntax';
			kind:
				| 'empty'
				| 'unexpectedChar'
				| 'lowercaseVariable'
				| 'unexpectedToken'
				| 'unexpectedEnd'
				| 'zeroCoefficient'
				| 'numberTooLarge';
			char: string | null;
			column: number;
			expected: ('operand' | 'operator' | 'closeParen' | 'end')[];
	  }
	| { code: 'missingWidth'; var: string }
	| { code: 'widthOutOfRange'; var: string; width: number; min: number; max: number }
	| { code: 'repeatedVariable'; var: string }
	| { code: 'constantTerm' }
	| { code: 'coefficientTooLarge' }
	| { code: 'tooManyInputBits'; total: number; limit: number }
	| { code: 'tooManyDots'; count: number; limit: number }
	| { code: 'badWidths' };

export type BuildResult = { ok: true; matrix: Matrix } | { ok: false; error: CoreError };

export interface Core {
	version: string;
	buildMatrix(src: string, widths: Record<string, number>): BuildResult;
}

export async function loadCore(): Promise<Core> {
	await init();
	return {
		version: coreVersion(),
		buildMatrix: (src, widths) => JSON.parse(wasmBuildMatrix(src, JSON.stringify(widths)))
	};
}
