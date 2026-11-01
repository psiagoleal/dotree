// Caminho relativo: app/web/src/lib/errors.spec.ts
// Every error code the core can return has a message in both languages (IDI-05).
import { describe, expect, it } from 'vitest';
import type { CoreError } from '#lib/core.ts';
import { errorMessage, warningMessage } from './errors';

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
