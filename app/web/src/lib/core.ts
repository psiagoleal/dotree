// Caminho relativo: app/web/src/lib/core.ts
// Loads the dotree-core WebAssembly module and types what it returns. The interface never
// reimplements domain rules (ADR 0001); it only calls into this module.
import init, {
	buildMatrix as wasmBuildMatrix,
	coreVersion,
	Session,
	SolutionMethod
} from '#lib/wasm/dotree.js';

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
	| { code: 'noVariables' }
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
	/** Why "close with CPA" is disabled, or null when it is available (CMP-10). */
	cpa: CpaBlock | null;
	/** Once complete: the point carrying each bit R_0..R_{W-1}, null for a constant 0 (CMP-12). */
	result: (number | null)[] | null;
	/** Cells on a path of maximum delay: numbers, and indices into `current.pending` (CMP-15). */
	critical: { cells: number[]; pending: number[] };
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

// ---------------------------------------------------------------- help (MT-13)

/** Only numbers, never a point or a cell (AJU-08). */
export type Hint =
	| { code: 'minLevelsLeft'; height: number; levels: number }
	| { code: 'tallestColumn'; column: number; height: number }
	| { code: 'idlePoints' }
	| { code: 'cpaReady' };

/** What Dadda's method would do now (AJU-04). */
export type NextMove =
	| { move: 'formCell'; kind: CellKind; points: number[] }
	| { move: 'concludeLevel' }
	| { move: 'closeWithCpa' }
	| { move: 'complete' };

export interface LowerBound {
	dots: number;
	outputWidth: number;
	value: number;
}

/** Revealed help steps (AJU-07). */
export interface HelpLog {
	hints: number;
	nextMoves: number;
	lowerBound: number;
	lowerBoundPath: number;
	solution: number;
	solutionPath: number;
	/** Times the solution replaced the circuit (AJU-09). */
	solutionApplied: number;
	/** The same three counts for Wallace's solution (AJU-10). */
	wallace: number;
	wallacePath: number;
	wallaceApplied: number;
}

/** The method of an automatic solution (AJU-05, AJU-10). */
export type Method = 'dadda' | 'wallace';

/** The proper name of each method, the same in every language. */
export const METHOD_NAME: Record<Method, string> = { dadda: 'Dadda', wallace: 'Wallace' };

const WASM_METHOD: Record<Method, SolutionMethod> = {
	dadda: SolutionMethod.Dadda,
	wallace: SolutionMethod.Wallace
};

// ---------------------------------------------------------------- exercise file (MT-15a)

/** The width challenge: the answer given, or null when the user gave up (EXP-10). */
export interface WidthReveal {
	answer: number | null;
	path: boolean;
}

/** How far a two-step help was revealed (GER-03). */
export type RevealStage = 'none' | 'result' | 'path';

export interface Revealed {
	width: WidthReveal | null;
	lowerBound: RevealStage;
	solution: RevealStage;
	wallace: RevealStage;
}

/** Why an exercise file cannot be opened (SAL-03). */
export type ExerciseError =
	| { code: 'malformed'; detail: string }
	| { code: 'notExercise' }
	| { code: 'oldSchema'; found: number }
	| { code: 'futureSchema'; found: number }
	| { code: 'badExpression' }
	| { code: 'inconsistentCell'; level: number; position: number }
	| { code: 'inconsistentLevel'; level: number }
	| { code: 'inconsistentReveals' };

export type RestoreResult =
	{ ok: true; session: CompressionSession } | { ok: false; error: ExerciseError };

// ---------------------------------------------------------------- diagram (MT-14a)

/** The input and AND block, a cell number, or result bit `R<j>`. */
export type DiagramEnd = 'inputs' | number | `R${number}`;

export interface DiagramBlock {
	cell: number;
	kind: CellKind;
	/** 0 is the input block; cells over level n sit in row n + 1. */
	row: number;
	column: number;
	/** Position among the blocks of the same row and column. */
	slot: number;
}

/** The block diagram of the concluded levels (spec 05). */
export interface Diagram {
	complete: boolean;
	outputWidth: number;
	rows: number;
	andGates: number;
	blocks: DiagramBlock[];
	edges: { from: DiagramEnd; to: DiagramEnd; label: string }[];
	/** Wire behind each R_j once complete; null is a constant 0. */
	outputs: (string | null)[];
	critical: number[];
	ands: { ident: string; factors: string[] }[];
}

/** One row of the wiring table (DIA-04). */
export interface WiringRow {
	cell: string;
	/** Null for the CPA. */
	level: number | null;
	column: number;
	inputs: (string | null)[];
	sum: string;
	sumResult: number | null;
	carry: string | null;
	carryResult: number | null;
}

export interface TableHeaders {
	cell: string;
	level: string;
	column: string;
	sum: string;
	carry: string;
}

// ------------------------------------------------- verification and VHDL (MT-14b)

/** First failing vector (VER-03). Values are decimal strings. */
export interface Counterexample {
	inputs: { var: string; value: string }[];
	expected: string;
	obtained: string;
	/** Result bits where the circuit disagrees with the expression. */
	differing: number[];
}

export interface VerificationProgress {
	checked: number;
	total: number;
	coverage: 'exhaustive' | 'sample';
	done: boolean;
	counterexample: Counterexample | null;
}

export interface VhdlFile {
	name: string;
	contents: string;
}

export type VhdlLanguage = 'pt-br' | 'en';

/** Whether the DE2 board files are included (spec 10, PLA-01). */
export type BoardFit =
	| { fits: true }
	| { fits: false; reason: 'tooManyInputs'; bits: number; max: number }
	| { fits: false; reason: 'tooWide'; width: number; max: number };

export type VhdlResult =
	| { ok: true; files: VhdlFile[]; board: BoardFit }
	| { ok: false; error: 'notComplete' | 'invalidName' };

/** A compression in progress, held by the core (MT-12a, MT-12b, MT-13, MT-14). */
export interface CompressionSession {
	snapshot(): Snapshot;
	formCell(ids: number[]): ActionResult;
	concludeLevel(): ActionResult;
	closeWithCpa(): ActionResult;
	goToLevel(level: number): ActionResult;
	undo(): boolean;
	redo(): boolean;
	/** Each help call is a reveal, counted by the core: call it only after confirmation. */
	hint(): Hint | null;
	nextMove(): NextMove;
	lowerBound(path: boolean): LowerBound;
	/** The solution of `method` from level 0, as a separate session; free it after use. */
	solution(path: boolean, method: Method): CompressionSession;
	/** Replaces the circuit with the solution of `method` as one undo step (AJU-09, AJU-10);
	 *  false before step 2 of that solution, or when the circuit already is it. Counted
	 *  when true. */
	applySolution(method: Method): boolean;
	helpLog(): HelpLog;
	diagram(): Diagram;
	wiringTable(): WiringRow[];
	wiringTableMarkdown(headers: TableHeaders): string;
	/** The complete circuit for the verification worker, or null while incomplete. */
	netlist(): string | null;
	vhdl(name: string, language: VhdlLanguage): VhdlResult;
	/** The exercise as a JSON file (SAL-02). */
	exercise(): string;
	/** What the exercise was opened with. */
	expression(): string;
	widths(): Record<string, number>;
	/** The width challenge was answered, or given up with null; only the first call counts. */
	revealWidth(answer: number | null): void;
	/** Step 2 of the width; false before step 1. */
	revealWidthPath(): boolean;
	revealed(): Revealed;
	/** Already revealed helps, again and uncounted; null before their reveal. */
	shownLowerBound(): LowerBound | null;
	shownSolution(method: Method): CompressionSession | null;
	/** Releases the WASM memory; the session is unusable afterwards. */
	free(): void;
}

export interface Core {
	version: string;
	buildMatrix(src: string, widths: Record<string, number>): BuildResult;
	/** Opens a compression session; call it only after `buildMatrix` succeeded. */
	openSession(src: string, widths: Record<string, number>): CompressionSession;
	/** Opens an exercise file, or says why it cannot (SAL-03). */
	restoreSession(file: string): RestoreResult;
}

function wrap(s: Session): CompressionSession {
	return {
		snapshot: () => JSON.parse(s.snapshot()),
		formCell: (ids) => JSON.parse(s.formCell(JSON.stringify(ids))),
		concludeLevel: () => JSON.parse(s.concludeLevel()),
		closeWithCpa: () => JSON.parse(s.closeWithCpa()),
		goToLevel: (level) => JSON.parse(s.goToLevel(level)),
		undo: () => s.undo(),
		redo: () => s.redo(),
		hint: () => JSON.parse(s.hint()).hint,
		nextMove: () => JSON.parse(s.nextMove()),
		lowerBound: (path) => JSON.parse(s.lowerBound(path)),
		solution: (path, method) => wrap(s.solution(path, WASM_METHOD[method])),
		applySolution: (method) => s.applySolution(WASM_METHOD[method]),
		helpLog: () => JSON.parse(s.helpLog()),
		diagram: () => JSON.parse(s.diagram()),
		wiringTable: () => JSON.parse(s.wiringTable()),
		wiringTableMarkdown: (headers) => s.wiringTableMarkdown(JSON.stringify(headers)),
		netlist: () => s.netlist() ?? null,
		vhdl: (name, language) => JSON.parse(s.vhdl(name, language)),
		exercise: () => s.exercise(),
		expression: () => s.expression(),
		widths: () => JSON.parse(s.widths()),
		revealWidth: (answer) => s.revealWidth(answer),
		revealWidthPath: () => s.revealWidthPath(),
		revealed: () => JSON.parse(s.revealed()),
		shownLowerBound: () => {
			const b = s.shownLowerBound();
			return b === undefined ? null : JSON.parse(b);
		},
		shownSolution: (method) => {
			const o = s.shownSolution(WASM_METHOD[method]);
			return o === undefined ? null : wrap(o);
		},
		free: () => s.free()
	};
}

export async function loadCore(): Promise<Core> {
	await init();
	return {
		version: coreVersion(),
		buildMatrix: (src, widths) => JSON.parse(wasmBuildMatrix(src, JSON.stringify(widths))),
		openSession: (src, widths) => wrap(Session.open(src, JSON.stringify(widths))),
		restoreSession: (file) => {
			try {
				return { ok: true, session: wrap(Session.restore(file)) };
			} catch (e) {
				return { ok: false, error: JSON.parse(String(e)) };
			}
		}
	};
}
