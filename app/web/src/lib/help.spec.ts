// Caminho relativo: app/web/src/lib/help.spec.ts
import { describe, expect, it } from 'vitest';
import { helpLogItems, hintMessage, moveMessage } from './help';
import type { HelpLog, Hint, LevelPoint } from './core';

const HINTS: Hint[] = [
	{ code: 'minLevelsLeft', height: 7, levels: 4 },
	{ code: 'tallestColumn', column: 1, height: 7 },
	{ code: 'idlePoints' },
	{ code: 'cpaReady' }
];

const pt = (id: number, column: number, signal: string): LevelPoint => ({
	id,
	column,
	ready: 0,
	signal,
	origin: 'input',
	term: 0
});

describe('hintMessage (spec 03, AJU-08)', () => {
	it('words the catalog as in spec 03', () => {
		expect(hintMessage(HINTS[0], 'pt-br')).toBe(
			'Com altura máxima 7, faltam no mínimo 4 níveis até o CPA.'
		);
		expect(hintMessage(HINTS[1], 'pt-br')).toBe(
			'A coluna 2^1 é a mais alta (7 pontos); o CPA aceita no máximo 3 na primeira coluna e 2 nas demais.'
		);
		expect(hintMessage(HINTS[3], 'pt-br')).toBe('Este nível já pode ser fechado com o CPA.');
	});

	it.each(['pt-br', 'en'] as const)('every hint is distinct and filled in %s', (locale) => {
		const texts = HINTS.map((h) => hintMessage(h, locale));
		for (const t of texts) expect(t).not.toMatch(/[{}]/);
		expect(new Set(texts).size).toBe(4);
	});
});

describe('moveMessage (AJU-04)', () => {
	const points = [pt(1, 1, 'a0_b1'), pt(2, 1, 'a1_b0'), pt(3, 0, 'a0_b0')];

	it('names the points of the cell and its column', () => {
		expect(moveMessage({ move: 'formCell', kind: 'ha', points: [1, 2] }, points, 'pt-br')).toBe(
			'Próxima jogada do método de Dadda: um HA com a0_b1 e a1_b0, na coluna 2^1. Os pontos estão destacados.'
		);
		expect(moveMessage({ move: 'formCell', kind: 'ha', points: [1, 2] }, points, 'en')).toContain(
			'a0_b1 and a1_b0'
		);
	});

	it('words the other moves', () => {
		expect(moveMessage({ move: 'concludeLevel' }, points, 'pt-br')).toBe(
			'Próxima jogada do método de Dadda: concluir o nível.'
		);
		expect(moveMessage({ move: 'closeWithCpa' }, points, 'pt-br')).toContain('CPA');
		expect(moveMessage({ move: 'complete' }, points, 'pt-br')).toBe('O circuito está completo.');
	});
});

describe('helpLogItems (AJU-07)', () => {
	const none: HelpLog = {
		hints: 0,
		nextMoves: 0,
		lowerBound: 0,
		lowerBoundPath: 0,
		solution: 0,
		solutionPath: 0
	};

	it('is empty when no help was used', () => {
		expect(helpLogItems(none, 'pt-br')).toEqual([]);
	});

	it('lists only the helps used, with both steps of the two-step ones', () => {
		expect(helpLogItems({ ...none, hints: 2, lowerBound: 1, lowerBoundPath: 1 }, 'pt-br')).toEqual([
			'dicas: 2',
			'limite inferior: 1 (conta: 1)'
		]);
		expect(helpLogItems({ ...none, nextMoves: 3, solution: 1 }, 'pt-br')).toEqual([
			'próximas jogadas: 3',
			'solução automática: 1 (nível a nível: 0)'
		]);
	});
});
