// Caminho relativo: app/web/src/lib/components/Counters.svelte.spec.ts
// CMP-20 (MT-32b, ADR 0005): the 7:3 and 15:4 counters in the compression panel, driving the
// real core with R1, whose column 2^1 has exactly 7 points.
import { beforeAll, describe, expect, it } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import Compression from './Compression.svelte';
import { loadCore, type Core } from '#lib/core.ts';
import { EXAMPLES } from '#lib/expression.ts';

let core: Core;
const r1 = EXAMPLES.find((e) => e.id === 'r1')!;
const R1_WIDTH = { outputWidth: 7, vMax: '79', widthRevealed: false };
const COLUMN_1 = ['a0_b1', 'a1_b0', 'c0_d1', 'c1_d0', 'e1', 'e0', 'f0'];

beforeAll(async () => {
	core = await loadCore();
});

const point = (signal: string, column: number) =>
	page
		.getByTestId('level-current')
		.getByRole('group', { name: new RegExp(`2\\^${column},`) })
		.getByRole('button', { name: signal, exact: true });

async function sevenOnColumn1() {
	render(Compression, {
		session: core.openSession(r1.expression, r1.widths),
		view: 'dots',
		...R1_WIDTH
	});
	for (const s of COLUMN_1) await point(s, 1).click();
	await expect.element(page.getByText('Seleção: 7')).toBeVisible();
	await page.getByRole('button', { name: 'Formar célula' }).click();
}

describe('counters in the compression (CMP-20)', () => {
	it('seven points of a column form a 7:3 counter, boxed and counted', async () => {
		await sevenOnColumn1();
		const box = page
			.getByTestId('level-current')
			.getByRole('group', { name: 'contador 7:3 pendente' });
		await expect.element(box).toBeVisible();
		expect(box.element().querySelectorAll('.dot')).toHaveLength(7);
		// A double border tells a counter from an FA (solid) and an HA (dashed).
		expect(getComputedStyle(box.element()).borderTopStyle).toBe('double');
		await expect.element(page.getByText('contador 7:3 formado.')).toBeInTheDocument();
		const metrics = page.getByTestId('metrics');
		await expect.element(metrics).toHaveTextContent(/7:3\s*1/);
		await expect.element(metrics).toHaveTextContent(/Área\s*4\s/);
		await expect.element(metrics).toHaveTextContent(/Atraso\s*3\s/);
		await expect.element(metrics).not.toHaveTextContent('15:4');
	});

	it('after the level, the outputs go to three columns and the tables show the counter', async () => {
		await sevenOnColumn1();
		await page.getByRole('button', { name: 'Concluir nível' }).click();
		await expect
			.element(page.getByTestId('level-0').getByRole('group', { name: 'contador 7:3, célula 1' }))
			.toBeVisible();
		await expect.element(point('s1', 1)).toBeVisible();
		await expect.element(point('k1', 2)).toBeVisible();
		await expect.element(point('k1_2', 3)).toBeVisible();
		const heights = page.getByTestId('height-table');
		await expect.element(heights.getByRole('columnheader', { name: '7:3' })).toBeVisible();
		const table = page.getByTestId('wiring-table');
		await expect.element(table.getByRole('columnheader', { name: 'Entradas' })).toBeVisible();
		await expect.element(table.getByRole('columnheader', { name: 'Bits 2 e 3' })).toBeVisible();
		await expect
			.element(table.getByRole('row', { name: /C73_1/ }))
			.toHaveTextContent('a0_b1, a1_b0, c0_d1, c1_d0, e1, e0, f0');
		await expect.element(table.getByRole('row', { name: /C73_1/ })).toHaveTextContent('k1_2');
		await expect.element(page.getByText('C73_1', { exact: true }).first()).toBeInTheDocument();
	});

	it('a counter takes its own cost in the panel; blank follows the FA network', async () => {
		await sevenOnColumn1();
		await page.getByTestId('costs-summary').click();
		const area = page.getByRole('textbox', { name: 'Área do 7:3', exact: true });
		await expect.element(area).toHaveValue('');
		await expect.element(area).toHaveAttribute('placeholder', 'rede: 4');
		await area.fill('2,5');
		await page.getByRole('button', { name: 'Aplicar custos' }).click();
		await expect.element(page.getByTestId('metrics')).toHaveTextContent(/Área\s*2,5/);
		await expect
			.element(page.getByTestId('costs-summary'))
			.toHaveTextContent(
				'FA: área 1, atraso 1 · HA: área 0,5, atraso 0,5 · 7:3: área 2,5, atraso 3'
			);
		await page.getByRole('button', { name: 'Restaurar o padrão' }).click();
		await expect.element(area).toHaveValue('');
		await expect.element(page.getByTestId('metrics')).toHaveTextContent(/Área\s*4\s/);
	});

	it('refuses a counter cost out of range, naming the field', async () => {
		await sevenOnColumn1();
		await page.getByTestId('costs-summary').click();
		await page.getByRole('textbox', { name: 'Atraso do 15:4', exact: true }).fill('0');
		await page.getByRole('button', { name: 'Aplicar custos' }).click();
		await expect
			.element(page.getByRole('alert'))
			.toHaveTextContent('Use números de 0,01 a 100, com até duas casas decimais: Atraso do 15:4.');
	});
});
