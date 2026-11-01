// Caminho relativo: app/web/src/lib/errors.ts
// Words the codes the core returns (IDI-05), in the current or a given language.
import { m } from '#lib/paraglide/messages.js';
import type { CoreError, CoreWarning, CpaBlock, Rejection } from '#lib/core.ts';

type Locale = 'pt-br' | 'en';

function expected(list: string[], locale?: Locale): string {
	const o = locale ? { locale } : {};
	const name = (e: string) =>
		({
			operand: m.expected_operand({}, o),
			operator: m.expected_operator({}, o),
			closeParen: m.expected_close_paren({}, o),
			end: m.expected_end({}, o)
		})[e] ?? e;
	return list.map(name).join(m.expected_or({}, o));
}

export function errorMessage(e: CoreError, locale?: Locale): string {
	const o = locale ? { locale } : {};
	switch (e.code) {
		case 'syntax': {
			const p = { char: e.char ?? '', column: e.column, expected: expected(e.expected, locale) };
			switch (e.kind) {
				case 'empty':
					return m.error_syntax_empty({}, o);
				case 'unexpectedChar':
					return m.error_syntax_unexpected_char(p, o);
				case 'lowercaseVariable':
					return m.error_syntax_lowercase({ ...p, upper: (e.char ?? '').toUpperCase() }, o);
				case 'unexpectedToken':
					return m.error_syntax_unexpected_token(p, o);
				case 'unexpectedEnd':
					return m.error_syntax_unexpected_end(p, o);
				case 'zeroCoefficient':
					return m.error_syntax_zero(p, o);
				case 'numberTooLarge':
					return m.error_syntax_number_too_large(p, o);
			}
			break;
		}
		case 'missingWidth':
			return m.error_missing_width({ var: e.var }, o);
		case 'widthOutOfRange':
			return m.error_width_out_of_range(e, o);
		case 'repeatedVariable':
			return m.error_repeated_variable({ var: e.var }, o);
		case 'constantTerm':
			return m.error_constant_term({}, o);
		case 'coefficientTooLarge':
			return m.error_coefficient_too_large({}, o);
		case 'tooManyInputBits':
			return m.error_too_many_input_bits(e, o);
		case 'tooManyDots':
			return m.error_too_many_dots(e, o);
		case 'badWidths':
			return m.error_bad_widths({}, o);
	}
	return m.error_bad_widths({}, o);
}

export function warningMessage(w: CoreWarning, locale?: Locale): string {
	return m.warning_unused_width({ var: w.var }, locale ? { locale } : {});
}

/** Refusals state the rule that was broken, never which points to pick (GER-02). */
export function rejectionMessage(r: Rejection, locale?: Locale): string {
	const o = locale ? { locale } : {};
	switch (r.code) {
		case 'mixedColumns':
			return m.reject_mixed_columns({}, o);
		case 'wrongCount':
			return m.reject_wrong_count({ count: r.count }, o);
		case 'alreadyInCell':
			return m.reject_already_in_cell({}, o);
		case 'notInCurrentLevel':
			return m.reject_not_in_current_level({}, o);
		case 'duplicateSelection':
			return m.reject_duplicate_selection({}, o);
		case 'emptyLevel':
			return m.reject_empty_level({}, o);
		case 'cpaUnavailable':
			return cpaBlockMessage(r.block, locale);
		case 'noSuchLevel':
			return m.reject_no_such_level({}, o);
		case 'badSelection':
			return m.reject_bad_selection({}, o);
	}
}

/** The condition "close with CPA" still lacks (CMP-10). */
export function cpaBlockMessage(b: CpaBlock, locale?: Locale): string {
	const o = locale ? { locale } : {};
	switch (b.code) {
		case 'columnTooTall':
			return m.cpa_block_column_too_tall({ column: b.column, height: b.height }, o);
		case 'threeNotFirst':
			return m.cpa_block_three_not_first({ column: b.column }, o);
		case 'nothingToAdd':
			return m.cpa_block_nothing_to_add({}, o);
		case 'pendingCells':
			return m.cpa_block_pending_cells({}, o);
	}
}
