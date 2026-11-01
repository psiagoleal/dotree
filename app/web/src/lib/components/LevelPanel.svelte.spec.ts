// Caminho relativo: app/web/src/lib/components/LevelPanel.svelte.spec.ts
import { describe, expect, it, vi } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import LevelPanel from './LevelPanel.svelte';
import type { LevelPoint } from '#lib/core.ts';

// 2A + B with A of 2 bits, B of 1 bit, plus one generated sum s1 in column 0.
const pt = (id: number, column: number, signal: string, term: number | null): LevelPoint => ({
	id,
	column,
	ready: 0,
	signal,
	origin: term === null ? 'sum' : 'input',
	term
});
const points = [pt(0, 1, 'a0', 0), pt(1, 2, 'a1', 0), pt(2, 0, 'b0', 1), pt(3, 0, 's1', null)];
const base = {
	title: 'Nível 0',
	testid: 'level-0',
	points,
	groups: [],
	top: 2,
	view: 'dots' as const
};

const weights = (c: HTMLElement) => [...c.querySelectorAll('.weight')].map((w) => w.textContent);

describe('LevelPanel', () => {
	it('draws the columns from `top` down to 2^0, labelled with their height (EXP-09, CMP-13)', async () => {
		const { container } = await render(LevelPanel, base);
		expect(weights(container)).toEqual(['22', '21', '20']);
		await expect
			.element(page.getByRole('group', { name: 'Coluna de peso 2^0, 2 pontos' }))
			.toBeVisible();
		const wider = await render(LevelPanel, { ...base, top: 3 });
		expect(weights(wider.container)).toEqual(['23', '22', '21', '20']);
	});

	it('colours inputs by term and cell outputs as generated (COR-01, COR-02)', async () => {
		const { container } = await render(LevelPanel, base);
		const dot = (s: string) => container.querySelector<HTMLElement>(`[data-signal="${s}"]`)!;
		expect(dot('a1').style.getPropertyValue('--c')).toBe('var(--dot-term-1)');
		expect(dot('b0').style.getPropertyValue('--c')).toBe('var(--dot-term-2)');
		expect(dot('s1').style.getPropertyValue('--c')).toBe('var(--dot-generated)');
		expect(dot('a1').getAttribute('aria-label')).toBe('a1');
	});

	it('writes signals with subscripts in the indexed view (VIS-01)', async () => {
		const { container } = await render(LevelPanel, { ...base, view: 'indexed' });
		const s1 = container.querySelector<HTMLElement>('[data-signal="s1"]')!;
		expect(s1.textContent).toBe('s1');
		expect(s1.querySelector('sub')?.textContent).toBe('1');
	});

	it('boxes each cell around its inputs: FA solid, HA dashed (CMP-05)', async () => {
		const groups = [{ kind: 'ha' as const, inputs: [3, 2], label: 'HA 1' }];
		const { container } = await render(LevelPanel, { ...base, groups });
		const box = page.getByRole('group', { name: 'HA 1' });
		await expect.element(box).toBeVisible();
		const inside = [...box.element().querySelectorAll<HTMLElement>('.dot')].map(
			(d) => d.dataset.signal
		);
		expect(inside).toEqual(['s1', 'b0']); // input order of the cell
		expect(box.element().classList.contains('ha')).toBe(true);
		expect(container.querySelectorAll('.cell')).toHaveLength(1);
	});

	it('read-only panels have no buttons', async () => {
		const { container } = await render(LevelPanel, base);
		expect(container.querySelectorAll('button')).toHaveLength(0);
	});

	it('in the current level, points are toggle buttons (GER-07)', async () => {
		const ontoggle = vi.fn();
		await render(LevelPanel, { ...base, selectable: true, selected: [1], ontoggle });
		const a1 = page.getByRole('button', { name: 'a1' });
		await expect.element(a1).toHaveAttribute('aria-pressed', 'true');
		await expect
			.element(page.getByRole('button', { name: 'b0' }))
			.toHaveAttribute('aria-pressed', 'false');
		await page.getByRole('button', { name: 'b0' }).click();
		expect(ontoggle).toHaveBeenCalledWith(2);
	});

	it('the result panel names each bit R_j in the result colour; an empty column is 0 (CMP-12, COR-02)', async () => {
		// 2A with A of 2 bits: R0 = 0, R1 = a0, R2 = a1.
		const { container } = await render(LevelPanel, {
			...base,
			title: 'Resultado',
			points: [pt(0, 1, 'a0', 0), pt(1, 2, 'a1', 0)],
			result: [null, 0, 1]
		});
		await expect
			.element(page.getByRole('group', { name: 'Coluna de peso 2^1, bit R1' }))
			.toBeVisible();
		await expect
			.element(page.getByRole('group', { name: 'Coluna de peso 2^0, bit R0 constante 0' }))
			.toHaveTextContent(/0\s*R0$/);
		const a0 = container.querySelector<HTMLElement>('[data-signal="a0"]')!;
		expect(a0.style.getPropertyValue('--c')).toBe('var(--dot-result)');
	});

	it('marks critical cells and dropped carries (CMP-15, CMP-09)', async () => {
		const { container } = await render(LevelPanel, {
			...base,
			groups: [
				{ kind: 'ha', inputs: [2, 3], label: 'HA 1', critical: true, dropped: true },
				{ kind: 'ha', inputs: [0], label: 'HA 2' }
			]
		});
		const box = page.getByRole('group', { name: 'HA 1' });
		await expect.element(box).toHaveClass(/critical/);
		await expect.element(box).toHaveTextContent('k=0');
		expect(container.querySelectorAll('.cell.critical')).toHaveLength(1);
		expect(container.querySelectorAll('.dropped')).toHaveLength(1);
	});
});
