// Caminho relativo: app/web/src/lib/methods.ts
// The words of each automatic solution (spec 03). Dadda and Wallace are named after their
// authors and share sentences with the name in them; the optimal solution of a height
// profile (AJU-12) has sentences of its own, so no language has to bend around a name.
import { METHOD_NAME, type Method } from '#lib/core.ts';
import { m } from '#lib/paraglide/messages.js';

export interface SummaryArgs {
	levels: number;
	fa: number;
	ha: number;
	area: string;
	delay: string;
}

export interface MethodTexts {
	ask: string;
	askPath: string;
	use: string;
	confirmTitle: string;
	confirmPathTitle: string;
	confirmUseTitle: string;
	confirmUseBody: string;
	title: string;
	rule: string;
	level: (level: number) => string;
	cpa: (level: number) => string;
	/** The last panel: the result, or the output in the profile for the optimum. */
	last: string;
	summary: (args: SummaryArgs, cpa: boolean) => string;
	applied: string;
	comparison: string;
	missing: string;
	heightsTitle: string;
	heightsExport: string;
}

export function methodTexts(method: Method): MethodTexts {
	if (method === 'optimal') {
		return {
			ask: m.optimal_ask(),
			askPath: m.optimal_ask_path(),
			use: m.optimal_use(),
			confirmTitle: m.optimal_confirm_title(),
			confirmPathTitle: m.optimal_confirm_path_title(),
			confirmUseTitle: m.optimal_confirm_use_title(),
			confirmUseBody: m.optimal_confirm_use_body(),
			title: m.optimal_title(),
			rule: m.optimal_rule(),
			level: (level) => m.optimal_level({ level }),
			// The optimum has no CPA level: its levels are all of FA and HA.
			cpa: (level) => m.optimal_level({ level }),
			last: m.optimal_output(),
			summary: (args) => m.optimal_summary(args),
			applied: m.optimal_applied(),
			comparison: m.comparison_optimal(),
			missing: m.comparison_missing_optimal(),
			heightsTitle: m.comparison_heights_title_optimal(),
			heightsExport: m.comparison_heights_export_optimal()
		};
	}
	const name = METHOD_NAME[method];
	const p = { method: name };
	return {
		ask: m.help_solution(p),
		askPath: m.help_solution_path(p),
		use: m.help_use_solution(p),
		confirmTitle: m.confirm_solution_title(p),
		confirmPathTitle: m.confirm_solution_path_title(p),
		confirmUseTitle: m.confirm_use_solution_title(p),
		confirmUseBody: m.confirm_use_solution_body(p),
		title: m.solution_title(p),
		rule: method === 'dadda' ? m.solution_rule_dadda() : m.solution_rule_wallace(),
		level: (level) => m.solution_level({ ...p, level }),
		cpa: (level) => m.solution_cpa({ ...p, level }),
		last: m.solution_result(p),
		summary: (args, cpa) =>
			cpa ? m.solution_summary_cpa({ ...p, ...args }) : m.solution_summary({ ...p, ...args }),
		applied: m.solution_applied(p),
		comparison: method === 'dadda' ? m.comparison_dadda() : m.comparison_wallace(),
		missing: m.comparison_missing(p),
		heightsTitle: m.comparison_heights_title(p),
		heightsExport: m.comparison_heights_export(p)
	};
}

/** The stage key of each method in `Session.revealed()`. */
export const STAGE_KEY = { dadda: 'solution', wallace: 'wallace', optimal: 'optimal' } as const;
