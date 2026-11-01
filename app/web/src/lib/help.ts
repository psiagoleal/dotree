// Caminho relativo: app/web/src/lib/help.ts
// Words the help the core returns (spec 03): hints, the next move and the help log. Only
// wording here; what to show and when is decided by the core and the confirmation.
import { m } from '#lib/paraglide/messages.js';
import type { HelpLog, Hint, LevelPoint, NextMove } from '#lib/core.ts';

type Locale = 'pt-br' | 'en';

export function hintMessage(h: Hint, locale?: Locale): string {
	const o = locale ? { locale } : {};
	switch (h.code) {
		case 'minLevelsLeft':
			return m.hint_min_levels_left({ height: h.height, levels: h.levels }, o);
		case 'tallestColumn':
			return m.hint_tallest_column({ column: h.column, height: h.height }, o);
		case 'idlePoints':
			return m.hint_idle_points({}, o);
		case 'cpaReady':
			return m.hint_cpa_ready({}, o);
	}
}

/** The next move, naming the points of a cell by their signals (from `points`). */
export function moveMessage(move: NextMove, points: LevelPoint[], locale?: Locale): string {
	const o = locale ? { locale } : {};
	switch (move.move) {
		case 'formCell': {
			const cell = move.points.map((id) => points.find((p) => p.id === id)!);
			const list = new Intl.ListFormat(locale === 'en' ? 'en' : 'pt-BR', {
				type: 'conjunction'
			}).format(cell.map((p) => p.signal));
			return m.move_form_cell(
				{ kind: move.kind.toUpperCase(), points: list, column: cell[0].column },
				o
			);
		}
		case 'concludeLevel':
			return m.move_conclude_level({}, o);
		case 'closeWithCpa':
			return m.move_close_with_cpa({}, o);
		case 'complete':
			return m.move_complete({}, o);
	}
}

/** The help log for the summary (AJU-07): one item per help used, or none. */
export function helpLogItems(log: HelpLog, locale?: Locale): string[] {
	const o = locale ? { locale } : {};
	const items: string[] = [];
	if (log.hints > 0) items.push(m.help_log_hints({ count: log.hints }, o));
	if (log.nextMoves > 0) items.push(m.help_log_next_moves({ count: log.nextMoves }, o));
	if (log.lowerBound + log.lowerBoundPath > 0)
		items.push(m.help_log_lower_bound({ result: log.lowerBound, path: log.lowerBoundPath }, o));
	if (log.solution + log.solutionPath > 0)
		items.push(m.help_log_solution({ result: log.solution, path: log.solutionPath }, o));
	return items;
}
