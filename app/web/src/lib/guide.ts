// Caminho relativo: app/web/src/lib/guide.ts
// The "How to use" text (GER-11), built from the catalog so both languages share one
// structure. Nothing here reads the open exercise: the guide can never reveal an answer.
import { m } from '#lib/paraglide/messages.js';
import { PROFILE_EXAMPLE } from '#lib/expression.ts';
import type { AppLocale } from '#lib/locale.ts';

export type GuideItem = { text: string; keys?: string[] };
export type GuideSection = {
	id: 'steps' | 'keys' | 'help' | 'limits' | 'references';
	title: string;
	intro?: string;
	items: GuideItem[];
};

export function guideSections(locale?: AppLocale): GuideSection[] {
	const o = locale ? { locale } : undefined;
	const items = (...texts: string[]) => texts.map((text) => ({ text }));
	return [
		{
			id: 'steps',
			title: m.guide_steps_title({}, o),
			items: items(
				m.guide_step_input({}, o),
				m.guide_step_profile({ example: PROFILE_EXAMPLE }, o),
				m.guide_step_width({}, o),
				m.guide_step_compress({}, o),
				m.guide_step_finish({}, o)
			)
		},
		{
			id: 'keys',
			title: m.guide_keys_title({}, o),
			intro: m.guide_keys_intro({}, o),
			items: [
				{ keys: ['Tab'], text: m.guide_key_tab({}, o) },
				{ keys: [m.guide_key_space({}, o)], text: m.guide_key_select({}, o) },
				{ keys: ['F'], text: m.guide_key_form({}, o) },
				{ keys: ['Esc'], text: m.guide_key_clear({}, o) },
				{ keys: ['Ctrl+Z', 'Ctrl+Y'], text: m.guide_key_undo({}, o) },
				{ keys: ['V'], text: m.guide_key_view({}, o) }
			]
		},
		{
			id: 'help',
			title: m.guide_help_title({}, o),
			intro: m.guide_help_intro({}, o),
			items: items(
				m.guide_help_width({}, o),
				m.guide_help_hint({}, o),
				m.guide_help_next({}, o),
				m.guide_help_bound({}, o),
				m.guide_help_solutions({}, o),
				m.guide_help_block({}, o),
				m.guide_help_free({}, o)
			)
		},
		{
			id: 'limits',
			title: m.guide_limits_title({}, o),
			items: items(
				m.guide_limit_sign({}, o),
				m.guide_limit_cells({}, o),
				m.guide_limit_size({}, o),
				m.guide_limit_cost({}, o),
				m.guide_limit_sample({}, o),
				m.guide_limit_profile({}, o),
				m.guide_limit_board({}, o),
				m.guide_limit_narrow({}, o)
			)
		},
		{
			id: 'references',
			title: m.guide_references_title({}, o),
			items: items(m.guide_ref_parhami({}, o), m.guide_ref_wallace({}, o), m.guide_ref_dadda({}, o))
		}
	];
}
