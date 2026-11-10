// Caminho relativo: app/web/src/lib/guide.spec.ts
// GER-11: the "How to use" text, in both languages, says nothing about any exercise.
import { describe, expect, it } from 'vitest';
import { guideSections } from './guide';

describe('guideSections (GER-11)', () => {
	it.each(['pt-br', 'en'] as const)('has the five parts, all filled, in %s', (locale) => {
		const sections = guideSections(locale);
		expect(sections.map((s) => s.id)).toEqual(['steps', 'keys', 'help', 'limits', 'references']);
		for (const s of sections) {
			expect(s.title.trim()).not.toBe('');
			expect(s.items.length).toBeGreaterThan(0);
			for (const item of s.items) expect(item.text.trim()).not.toBe('');
		}
	});

	it('lists every page shortcut with its keys', () => {
		const keys = guideSections('en').find((s) => s.id === 'keys')!;
		expect(keys.items.flatMap((i) => i.keys ?? [])).toEqual(
			expect.arrayContaining(['Tab', 'Space', 'F', 'Esc', 'Ctrl+Z', 'Ctrl+Y', 'V'])
		);
		expect(keys.items.every((i) => (i.keys ?? []).length > 0)).toBe(true);
	});

	it('names every help and says what does not count', () => {
		const help = guideSections('pt-br').find((s) => s.id === 'help')!;
		const text = help.items.map((i) => i.text).join('\n');
		for (const name of [
			'Largura',
			'Dica',
			'Próxima jogada',
			'Limite inferior',
			'Dadda',
			'Wallace',
			'ótima',
			'Nome do bloco',
			'Não contam'
		])
			expect(text).toContain(name);
		expect(help.intro).toMatch(/duas etapas/);
	});

	it('cites only public books and papers (ADR 0004)', () => {
		for (const locale of ['pt-br', 'en'] as const) {
			const refs = guideSections(locale).find((s) => s.id === 'references')!;
			const text = refs.items.map((i) => i.text).join('\n');
			expect(text).toMatch(/Parhami/);
			expect(text).toMatch(/Wallace.*1964/);
			expect(text).toMatch(/Dadda.*1965/);
		}
		const all = (['pt-br', 'en'] as const)
			.flatMap((l) => guideSections(l))
			.flatMap((s) => [s.title, s.intro ?? '', ...s.items.map((i) => i.text)])
			.join('\n');
		expect(all).not.toMatch(
			/gabarito|answer key|\baulas?\b|lecture|\blab\b|laborat|slides?|professor/i
		);
	});
});
