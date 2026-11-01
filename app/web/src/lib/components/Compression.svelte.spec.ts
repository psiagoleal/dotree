// Caminho relativo: app/web/src/lib/components/Compression.svelte.spec.ts
// Drives the real core (WASM) through the compression panel with R1 (spec 00).
import { beforeAll, describe, expect, it } from 'vitest';
import { page, userEvent } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import Compression from './Compression.svelte';
import { loadCore, type Core } from '#lib/core.ts';
import { EXAMPLES } from '#lib/expression.ts';

let core: Core;
const r1 = EXAMPLES.find((e) => e.id === 'r1')!;

beforeAll(async () => {
	core = await loadCore();
});

async function open(view: 'dots' | 'indexed' = 'dots') {
	const session = core.openSession(r1.expression, r1.widths);
	return render(Compression, { session, view, width: null });
}

/** The point with this signal in column `column` of the current level. */
const point = (signal: string, column: number) =>
	page
		.getByTestId('level-current')
		.getByRole('group', { name: new RegExp(`2\\^${column},`) })
		.getByRole('button', { name: signal, exact: true });

describe('Compression (CMP-01, CMP-02, CMP-05, CMP-13 to CMP-15, GER-02)', () => {
	it('selection survives switching the view (VIS-02)', async () => {
		const { rerender } = await open();
		await point('a0_b2', 2).click();
		await point('a1_b1', 2).click();
		await expect.element(page.getByText('Seleção: 2')).toBeVisible();
		await rerender({ view: 'indexed' });
		await expect.element(point('a0_b2', 2)).toHaveAttribute('aria-pressed', 'true');
		await expect.element(point('a0_b2', 2)).toHaveTextContent('a0b2');
		await expect.element(point('a1_b1', 2)).toHaveAttribute('aria-pressed', 'true');
	});

	it('forming a cell boxes its points and counts it at once', async () => {
		await open();
		await point('a0_b2', 2).click();
		await point('a1_b1', 2).click();
		await page.getByRole('button', { name: 'Formar célula' }).click();
		const box = page.getByTestId('level-current').getByRole('group', { name: 'HA pendente' });
		await expect.element(box).toBeVisible();
		expect(box.element().querySelectorAll('.dot')).toHaveLength(2);
		await expect.element(page.getByTestId('metrics')).toHaveTextContent(/HA\s*1/);
		await expect.element(page.getByTestId('metrics')).toHaveTextContent(/Área\s*0,5/);
		await expect.element(page.getByTestId('metrics')).toHaveTextContent(/Atraso\s*0,5/);
		await expect.element(page.getByText('Seleção: 0')).toBeVisible();
	});

	it('a refusal states the rule and keeps the selection (GER-02, CMP-07)', async () => {
		await open();
		await point('a0_b0', 0).click();
		await page.getByRole('button', { name: 'Formar célula' }).click();
		await expect
			.element(page.getByRole('alert'))
			.toHaveTextContent('Um FA usa 3 pontos; um HA usa 2. A seleção tem 1.');
		await expect.element(point('a0_b0', 0)).toHaveAttribute('aria-pressed', 'true');
		expect(
			page
				.getByTestId('level-current')
				.getByRole('group', { name: /pendente/ })
				.elements()
		).toHaveLength(0);
	});

	it('concluding a level adds a panel; the cell gets its number (CMP-05)', async () => {
		await open();
		await page.getByRole('button', { name: 'Concluir nível' }).click();
		await expect
			.element(page.getByRole('alert'))
			.toHaveTextContent('Um nível precisa de pelo menos uma célula.');

		await point('a0_b1', 1).click();
		await point('a1_b0', 1).click();
		await point('c0_d1', 1).click();
		await page.getByRole('button', { name: 'Formar célula' }).click();
		await page.getByRole('button', { name: 'Concluir nível' }).click();

		await expect
			.element(page.getByTestId('level-0').getByRole('group', { name: 'FA 1' }))
			.toBeVisible();
		await expect.element(page.getByRole('heading', { name: 'Nível 1 (atual)' })).toBeVisible();
		await expect.element(point('s1', 1)).toBeVisible();
		await expect.element(point('k1', 2)).toBeVisible();
		await expect
			.element(page.getByTestId('level-current').getByRole('group', { name: /2\^1,/ }))
			.toHaveAccessibleName('Coluna de peso 2^1, 5 pontos');
	});

	it('keyboard: F forms, Ctrl+Z undoes, Ctrl+Y redoes, Esc clears (GER-07, CMP-03)', async () => {
		await open();
		const undo = page.getByRole('button', { name: 'Desfazer' });
		await expect.element(undo).toBeDisabled();
		await point('a0_b2', 2).click();
		await point('a1_b1', 2).click();
		await userEvent.keyboard('f');
		const ha = page.getByTestId('level-current').getByRole('group', { name: 'HA pendente' });
		await expect.element(ha).toBeVisible();
		await expect.element(undo).toBeEnabled();

		await userEvent.keyboard('{Control>}z{/Control}');
		await expect.element(ha).not.toBeInTheDocument();
		await userEvent.keyboard('{Control>}y{/Control}');
		await expect.element(ha).toBeVisible();

		await point('a0_b0', 0).click();
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByText('Seleção: 0')).toBeVisible();
	});
});
