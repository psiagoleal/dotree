// Caminho relativo: app/web/src/lib/components/CostSettings.svelte.spec.ts
// GER-12 (MT-45): the cost panel inside the compression, driving the real core with R1.
import { beforeAll, describe, expect, it } from 'vitest';
import { page, userEvent } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import Compression from './Compression.svelte';
import { DEFAULT_COSTS, loadCore, type Core } from '#lib/core.ts';
import { EXAMPLES } from '#lib/expression.ts';
import { formOn, r2Levels } from '#lib/testing/r2.ts';

let core: Core;
const r1 = EXAMPLES.find((e) => e.id === 'r1')!;
const R1_WIDTH = { outputWidth: 7, vMax: '79', widthRevealed: false };

beforeAll(async () => {
	core = await loadCore();
});

const metrics = () => page.getByTestId('metrics');
const field = (name: string) => page.getByRole('textbox', { name, exact: true });

async function setField(name: string, value: string) {
	await field(name).fill(value);
}

async function openCosts() {
	await page.getByTestId('costs-summary').click();
}

describe('CostSettings (GER-12)', () => {
	it('starts with the default costs, shown in the summary', async () => {
		render(Compression, {
			session: core.openSession(r1.expression, r1.widths),
			view: 'dots',
			...R1_WIDTH
		});
		await expect
			.element(page.getByTestId('costs-summary'))
			.toHaveTextContent('FA: área 1, atraso 1 · HA: área 0,5, atraso 0,5');
		await openCosts();
		await expect.element(field('Área do HA')).toHaveValue('0,5');
		await expect.element(page.getByRole('button', { name: 'Restaurar o padrão' })).toBeDisabled();
	});

	it('R2 under FA 1,2 / 1 and HA 0,7 / 0,5: 13 FA + 4 HA = 18,4 A_FA; restore gives 15 back', async () => {
		// R2: 13 FA, 4 HA, 15 A_FA, 6 T_FA (spec 00); the delay keeps FA 1 and HA 0.5.
		const session = r2Levels(core, 3);
		session.closeWithCpa();
		render(Compression, { session, view: 'dots', ...R1_WIDTH });
		await expect.element(metrics()).toHaveTextContent(/Área\s*15\s/);
		await openCosts();
		await setField('Área do FA', '1,2');
		await setField('Área do HA', '0.7');
		await page.getByRole('button', { name: 'Aplicar custos' }).click();
		await expect.element(metrics()).toHaveTextContent(/Área\s*18,4/);
		await expect.element(metrics()).toHaveTextContent(/Atraso\s*6\s/);
		expect(session.costs()).toEqual({ faArea: 120, faDelay: 100, haArea: 70, haDelay: 50 });
		await expect.element(page.getByText('alterados')).toBeVisible();
		// Not an undo step: undo removes the CPA, not the costs.
		await page.getByRole('button', { name: 'Desfazer' }).click();
		expect(session.costs().faArea).toBe(120);
		await page.getByRole('button', { name: 'Refazer' }).click();

		await page.getByRole('button', { name: 'Restaurar o padrão' }).click();
		await expect.element(metrics()).toHaveTextContent(/Área\s*15\s/);
		await expect.element(field('Área do FA')).toHaveValue('1');
		expect(session.costs()).toEqual(DEFAULT_COSTS);
	});

	it('a faster FA moves the critical path to the HA branch (hand case of the core)', async () => {
		// A + B + C + 2(D + E), widths 1: FA 1 on column 0, HA 2 on column 1, then the CPA's
		// HA 3 and HA 4. Default: 2 T_FA through FA 1. With FA delay 0,4: HA 3 waits for s2
		// (0,5), ends at 1, and HA 4 at 1,5; the critical path now goes through HA 2.
		const session = core.openSession('A+B+C+2(D+E)', { A: 1, B: 1, C: 1, D: 1, E: 1 });
		formOn(session, 0, ['a0', 'b0', 'c0']);
		formOn(session, 1, ['d0', 'e0']);
		session.concludeLevel();
		session.closeWithCpa();
		render(Compression, { session, view: 'dots', outputWidth: 3, vMax: '7', widthRevealed: true });
		await expect.element(metrics()).toHaveTextContent(/Atraso\s*2\s/);
		expect(session.snapshot().critical.cells).toEqual([1, 3, 4]);
		await openCosts();
		await setField('Atraso do FA', '0,4');
		await userEvent.keyboard('{Enter}');
		await expect.element(metrics()).toHaveTextContent(/Atraso\s*1,5/);
		expect(session.snapshot().critical.cells).toEqual([2, 3, 4]);
		await page.getByRole('button', { name: 'Destacar caminho crítico' }).click();
		await expect
			.element(page.getByRole('group', { name: 'HA 2, no caminho crítico' }))
			.toBeVisible();
	});

	it('refuses a cost out of range or with three decimals, naming the field, and changes nothing', async () => {
		const session = core.openSession(r1.expression, r1.widths);
		render(Compression, { session, view: 'dots', ...R1_WIDTH });
		await openCosts();
		await setField('Área do FA', '0');
		await setField('Atraso do HA', '0,125');
		await page.getByRole('button', { name: 'Aplicar custos' }).click();
		await expect
			.element(page.getByRole('alert'))
			.toHaveTextContent(
				'Use números de 0,01 a 100, com até duas casas decimais: Área do FA, Atraso do HA.'
			);
		await expect.element(field('Área do FA')).toHaveAttribute('aria-invalid', 'true');
		expect(session.costs()).toEqual(DEFAULT_COSTS);
		expect(session.helpLog().hints).toBe(0);
	});
});
