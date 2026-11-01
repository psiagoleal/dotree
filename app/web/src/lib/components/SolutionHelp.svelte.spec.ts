// Caminho relativo: app/web/src/lib/components/SolutionHelp.svelte.spec.ts
// Lower bound and automatic solution on R1 (spec 00), in two steps (GER-03).
import { beforeAll, describe, expect, it, vi } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import SolutionHelp from './SolutionHelp.svelte';
import { loadCore, type Core } from '#lib/core.ts';
import { EXAMPLES } from '#lib/expression.ts';

let core: Core;
const r1 = EXAMPLES.find((e) => e.id === 'r1')!;

beforeAll(async () => {
	core = await loadCore();
});

async function open() {
	const session = core.openSession(r1.expression, r1.widths);
	const onhelp = vi.fn();
	const { container } = await render(SolutionHelp, { session, view: 'dots', onhelp });
	return { session, onhelp, container };
}

const show = () => page.getByRole('dialog').getByRole('button', { name: 'Mostrar' });
const keepTrying = () => page.getByRole('button', { name: 'Vou tentar mais um pouco' });

describe('SolutionHelp (GER-03, AJU-03, AJU-05, AJU-06)', () => {
	it('lower bound: the number first, the reasoning only on a second request', async () => {
		const { onhelp } = await open();
		const pathButton = page.getByRole('button', { name: 'Ver a conta do limite' });
		await expect.element(pathButton).not.toBeInTheDocument();

		await page.getByRole('button', { name: 'Limite inferior' }).click();
		await keepTrying().click();
		await expect.element(page.getByTestId('lower-bound')).not.toBeInTheDocument();

		await page.getByRole('button', { name: 'Limite inferior' }).click();
		await show().click();
		await expect
			.element(page.getByTestId('lower-bound'))
			.toHaveTextContent(/^Limite inferior: 13\./);
		await expect.element(page.getByTestId('lower-bound-path')).not.toBeInTheDocument();

		await pathButton.click();
		await expect.element(keepTrying()).toHaveFocus();
		await show().click();
		await expect.element(page.getByTestId('lower-bound-path')).toHaveTextContent('20 − 7 = 13');
		expect(onhelp).toHaveBeenCalledTimes(2);
	});

	it('solution: final metrics first, the levels on a second request; the circuit is untouched', async () => {
		const { session, container } = await open();
		const before = JSON.stringify(session.snapshot());

		await page.getByRole('button', { name: 'Solução automática' }).click();
		await show().click();
		await expect
			.element(page.getByTestId('solution-summary'))
			.toHaveTextContent(/^Solução de Dadda: 4 níveis e o CPA; 13 FA, 8 HA; área 17 A_FA; atraso/);
		expect(container.querySelectorAll('section.panel')).toHaveLength(0);

		await page.getByRole('button', { name: 'Ver a solução nível a nível' }).click();
		await show().click();
		await expect
			.element(page.getByRole('heading', { name: 'Solução, nível 4: CPA' }))
			.toBeVisible();
		await expect.element(page.getByRole('heading', { name: 'Solução, resultado' })).toBeVisible();
		expect(container.querySelectorAll('section.panel')).toHaveLength(6);
		expect(JSON.stringify(session.snapshot())).toBe(before);
		expect(session.helpLog()).toMatchObject({ solution: 1, solutionPath: 1 });
	});
});
