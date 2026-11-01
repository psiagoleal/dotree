// Caminho relativo: app/web/src/lib/core.ts
// Loads the dotree-core WebAssembly module and types what it returns. The interface never
// reimplements domain rules (ADR 0001); it only calls into this module.
import init, { buildMatrix as wasmBuildMatrix, coreVersion, Session } from '#lib/wasm/dotree.js';

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

// ---------------------------------------------------------------- compression (MT-12a)
// Times (`ready`, `delay`) and `area` are in half units: FA = 2, HA = 1.

export type CellKind = 'fa' | 'ha';

export interface LevelPoint {
	id: number;
	column: number;
	ready: number;
	signal: string;
	origin: 'input' | 'sum' | 'carry';
	/** Term of an input point (its colour); null for cell outputs. */
	term: number | null;
}

export interface PendingCell {
	kind: CellKind;
	column: number;
	/** Point ids, in canonical order. */
	inputs: number[];
	ready: number;
}

export interface Cell extends PendingCell {
	number: number;
	sum: number;
	/** Null when the carry fell beyond the output and was dropped (CMP-09). */
	carry: number | null;
}

export interface Level {
	points: LevelPoint[];
	cells: Cell[];
	cpa: boolean;
}

export interface Snapshot {
	/** Index of the current level (0 = the initial matrix). */
	level: number;
	/** Concluded levels, oldest first. */
	levels: Level[];
	current: { points: LevelPoint[]; pending: PendingCell[] };
	heights: number[];
	metrics: { fa: number; ha: number; andGates: number; area: number; delay: number };
	canUndo: boolean;
	canRedo: boolean;
}

export type CpaBlock =
	| { code: 'columnTooTall'; column: number; height: number }
	| { code: 'threeNotFirst'; column: number }
	| { code: 'nothingToAdd' }
	| { code: 'pendingCells' };

export type Rejection =
	| { code: 'mixedColumns' }
	| { code: 'wrongCount'; count: number }
	| { code: 'alreadyInCell'; point: number }
	| { code: 'notInCurrentLevel'; point: number }
	| { code: 'duplicateSelection'; point: number }
	| { code: 'emptyLevel' }
	| { code: 'cpaUnavailable'; block: CpaBlock }
	| { code: 'noSuchLevel'; level: number }
	| { code: 'badSelection' };

export type ActionResult = { ok: true; kind?: CellKind } | { ok: false; rejection: Rejection };

/** A compression in progress, held by the core (MT-12a). */
export interface CompressionSession {
	snapshot(): Snapshot;
	formCell(ids: number[]): ActionResult;
	concludeLevel(): ActionResult;
	undo(): boolean;
	redo(): boolean;
	/** Releases the WASM memory; the session is unusable afterwards. */
	free(): void;
}

export interface Core {
	version: string;
	buildMatrix(src: string, widths: Record<string, number>): BuildResult;
	/** Opens a compression session; call it only after `buildMatrix` succeeded. */
	openSession(src: string, widths: Record<string, number>): CompressionSession;
}

function wrap(s: Session): CompressionSession {
	return {
		snapshot: () => JSON.parse(s.snapshot()),
		formCell: (ids) => JSON.parse(s.formCell(JSON.stringify(ids))),
		concludeLevel: () => JSON.parse(s.concludeLevel()),
		undo: () => s.undo(),
		redo: () => s.redo(),
		free: () => s.free()
	};
}

export async function loadCore(): Promise<Core> {
	await init();
	return {
		version: coreVersion(),
		buildMatrix: (src, widths) => JSON.parse(wasmBuildMatrix(src, JSON.stringify(widths))),
		openSession: (src, widths) => wrap(Session.open(src, JSON.stringify(widths)))
	};
}
