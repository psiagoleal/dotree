// Caminho relativo: app/web/src/lib/errors.spec.ts
// Every error code the core can return has a message in both languages (IDI-05).
import { describe, expect, it } from 'vitest';
import type { CoreError, CpaBlock, Rejection } from '#lib/core.ts';
import { cpaBlockMessage, errorMessage, rejectionMessage, warningMessage } from './errors';

const ALL: CoreError[] = [
	...(
		[
			'empty',
			'unexpectedChar',
			'lowercaseVariable',
			'unexpectedToken',
			'unexpectedEnd',
			'zeroCoefficient',
			'numberTooLarge'
		] as const
	).map(
		(kind) =>
			({ code: 'syntax', kind, char: 'a', column: 3, expected: ['operand', 'end'] }) as CoreError
	),
	{ code: 'missingWidth', var: 'B' },
	{ code: 'widthOutOfRange', var: 'A', width: 17, min: 1, max: 16 },
	{ code: 'repeatedVariable', var: 'A' },
	{ code: 'constantTerm' },
	{ code: 'coefficientTooLarge' },
	{ code: 'tooManyInputBits', total: 33, limit: 32 },
	{ code: 'tooManyDots', count: 257, limit: 256 },
	{ code: 'badWidths' }
];

describe('errorMessage', () => {
	it.each(['pt-br', 'en'] as const)(
		'words every code in %s, without placeholders left',
		(locale) => {
			const texts = ALL.map((e) => errorMessage(e, locale));
			for (const t of texts) {
				expect(t.length).toBeGreaterThan(5);
				expect(t).not.toMatch(/[{}]/);
			}
			expect(new Set(texts).size).toBe(texts.length);
		}
	);

	it('names the column and what was expected (EXP-05)', () => {
		const e: CoreError = {
			code: 'syntax',
			kind: 'unexpectedEnd',
			char: null,
			column: 3,
			expected: ['operand']
		};
		expect(errorMessage(e, 'pt-br')).toBe(
			'A expressão terminou antes da hora (coluna 3). Esperado: uma variável, um número ou “(”.'
		);
		expect(errorMessage(e, 'en')).toBe(
			'The expression ended too early (column 3). Expected: a variable, a number or “(”.'
		);
	});

	it('suggests the upper-case letter', () => {
		const e: CoreError = {
			code: 'syntax',
			kind: 'lowercaseVariable',
			char: 'a',
			column: 1,
			expected: ['operand']
		};
		expect(errorMessage(e, 'pt-br')).toContain('deveria ser “A”');
	});

	it('words the unused-width warning (EXP-11)', () => {
		expect(warningMessage({ code: 'unusedWidth', var: 'Z' }, 'en')).toContain('Z');
	});
});

const REJECTIONS: Rejection[] = [
	{ code: 'mixedColumns' },
	{ code: 'wrongCount', count: 4 },
	{ code: 'alreadyInCell', point: 3 },
	{ code: 'notInCurrentLevel', point: 3 },
	{ code: 'duplicateSelection', point: 3 },
	{ code: 'emptyLevel' },
	{ code: 'cpaUnavailable', block: { code: 'nothingToAdd' } },
	{ code: 'noSuchLevel', level: 9 },
	{ code: 'badSelection' }
];

describe('rejectionMessage (GER-02)', () => {
	it.each(['pt-br', 'en'] as const)('words every refusal in %s', (locale) => {
		const texts = REJECTIONS.map((r) => rejectionMessage(r, locale));
		for (const t of texts) {
			expect(t.length).toBeGreaterThan(5);
			expect(t).not.toMatch(/[{}]/);
		}
		expect(new Set(texts).size).toBe(texts.length);
	});

	it('states the rule from spec 02, never which points to pick', () => {
		expect(rejectionMessage({ code: 'mixedColumns' }, 'pt-br')).toBe(
			'Uma célula soma bits do mesmo peso.'
		);
		expect(rejectionMessage({ code: 'wrongCount', count: 4 }, 'pt-br')).toBe(
			'Um FA usa 3 pontos; um HA usa 2. A seleção tem 4.'
		);
		expect(rejectionMessage({ code: 'alreadyInCell', point: 3 }, 'pt-br')).toBe(
			'Cada ponto entra em uma célula só por nível.'
		);
		expect(rejectionMessage({ code: 'emptyLevel' }, 'pt-br')).toBe(
			'Um nível precisa de pelo menos uma célula.'
		);
	});
});

const BLOCKS: CpaBlock[] = [
	{ code: 'columnTooTall', column: 0, height: 5 },
	{ code: 'threeNotFirst', column: 3 },
	{ code: 'nothingToAdd' },
	{ code: 'pendingCells' }
];

describe('cpaBlockMessage (CMP-10)', () => {
	it.each(['pt-br', 'en'] as const)('words every block in %s', (locale) => {
		const texts = BLOCKS.map((b) => cpaBlockMessage(b, locale));
		for (const t of texts) expect(t).not.toMatch(/[{}]/);
		expect(new Set(texts).size).toBe(texts.length);
	});

	it('names the condition that is missing', () => {
		expect(cpaBlockMessage(BLOCKS[0], 'pt-br')).toBe(
			'A coluna 2^0 tem 5 pontos; o CPA aceita no máximo 3.'
		);
		expect(cpaBlockMessage(BLOCKS[1], 'pt-br')).toContain('2^3');
	});

	it('a refused CPA says the same as the disabled button', () => {
		expect(rejectionMessage({ code: 'cpaUnavailable', block: BLOCKS[3] }, 'pt-br')).toBe(
			cpaBlockMessage(BLOCKS[3], 'pt-br')
		);
	});
});
