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

async function open(readOnly = false) {
	const session = core.openSession(r1.expression, r1.widths);
	const onhelp = vi.fn();
	const onapply = vi.fn();
	const { container } = await render(SolutionHelp, {
		session,
		view: 'dots',
		onhelp,
		onapply,
		readOnly
	});
	return { session, onhelp, onapply, container };
}

const useIt = () => page.getByRole('button', { name: 'Usar a solução de Dadda', exact: true });
const replace = () => page.getByRole('dialog').getByRole('button', { name: 'Substituir' });

async function revealPath() {
	await page.getByRole('button', { name: 'Solução de Dadda', exact: true }).click();
	await show().click();
	await page.getByRole('button', { name: 'Ver a solução de Dadda nível a nível' }).click();
	await show().click();
}

const show = () => page.getByRole('dialog').getByRole('button', { name: 'Mostrar' });
const keepTrying = () => page.getByRole('button', { name: 'Vou tentar mais um pouco' });

describe('SolutionHelp (GER-03, AJU-03, AJU-05, AJU-06, AJU-10)', () => {
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

		await page.getByRole('button', { name: 'Solução de Dadda', exact: true }).click();
		await show().click();
		await expect
			.element(page.getByTestId('solution-summary'))
			.toHaveTextContent(/^Solução de Dadda: 4 níveis e o CPA; 13 FA, 8 HA; área 17 A_FA; atraso/);
		expect(container.querySelectorAll('section.panel')).toHaveLength(0);

		await page.getByRole('button', { name: 'Ver a solução de Dadda nível a nível' }).click();
		await show().click();
		await expect
			.element(page.getByRole('heading', { name: 'Solução de Dadda, nível 4: CPA' }))
			.toBeVisible();
		await expect
			.element(page.getByRole('heading', { name: 'Solução de Dadda, resultado' }))
			.toBeVisible();
		expect(container.querySelectorAll('section.panel')).toHaveLength(6);
		expect(JSON.stringify(session.snapshot())).toBe(before);
		expect(session.helpLog()).toMatchObject({ solution: 1, solutionPath: 1 });
	});

	it('a reopened exercise shows the helps already revealed, without counting them (SAL-02)', async () => {
		const first = core.openSession(r1.expression, r1.widths);
		first.lowerBound(false);
		first.lowerBound(true);
		first.solution(false, 'dadda').free();
		first.solution(false, 'wallace').free();
		const reopened = core.restoreSession(first.exercise());
		first.free();
		if (!reopened.ok) throw new Error(reopened.error.code);
		const session = reopened.session;
		const log = session.helpLog();
		const onhelp = vi.fn();
		await render(SolutionHelp, { session, view: 'dots', onhelp, onapply: vi.fn() });

		await expect.element(page.getByTestId('lower-bound-path')).toHaveTextContent('20 − 7 = 13');
		await expect.element(page.getByTestId('solution-summary')).toBeVisible();
		await expect.element(page.getByTestId('solution')).not.toBeInTheDocument();
		await expect
			.element(page.getByTestId('wallace-summary'))
			.toHaveTextContent(/^Solução de Wallace: /);
		await expect
			.element(page.getByRole('button', { name: 'Ver a solução de Dadda nível a nível' }))
			.toBeVisible();
		expect(session.helpLog()).toEqual(log);
		expect(onhelp).not.toHaveBeenCalled();
	});

	it('using the solution: offered only after the levels, asked first, then replaces the circuit (AJU-09)', async () => {
		const { session, onapply } = await open();
		await expect.element(useIt()).not.toBeInTheDocument();
		await page.getByRole('button', { name: 'Solução de Dadda', exact: true }).click();
		await show().click();
		await expect.element(useIt()).not.toBeInTheDocument();
		await page.getByRole('button', { name: 'Ver a solução de Dadda nível a nível' }).click();
		await show().click();

		await useIt().click();
		await expect.element(keepTrying()).toHaveFocus();
		await keepTrying().click();
		expect(onapply).not.toHaveBeenCalled();
		expect(session.snapshot().result).toBeNull();

		await useIt().click();
		await replace().click();
		expect(onapply).toHaveBeenCalledTimes(1);
		expect(onapply).toHaveBeenCalledWith('dadda');
		const snap = session.snapshot();
		expect(snap.metrics).toMatchObject({ fa: 13, ha: 8 });
		expect(snap.result).not.toBeNull();
		expect(session.helpLog()).toMatchObject({ solutionApplied: 1 });
	});

	it('read-only mode does not offer to use the solution (GER-10)', async () => {
		await open(true);
		await revealPath();
		await expect.element(page.getByTestId('solution')).toBeVisible();
		await expect.element(useIt()).not.toBeInTheDocument();
	});

	it("Wallace's solution: its own two steps, with its rule, apart from Dadda's (AJU-10)", async () => {
		const { session, container } = await open();
		const before = JSON.stringify(session.snapshot());

		await page.getByRole('button', { name: 'Solução de Wallace' }).click();
		await expect.element(keepTrying()).toHaveFocus();
		await show().click();
		await expect
			.element(page.getByTestId('wallace-summary'))
			.toHaveTextContent(
				/^Solução de Wallace: 4 níveis e o CPA; 13 FA, 3 HA; área 14,5 A_FA; atraso/
			);
		await expect.element(page.getByTestId('solution-summary')).not.toBeInTheDocument();
		await expect
			.element(page.getByRole('button', { name: 'Ver a solução de Dadda nível a nível' }))
			.not.toBeInTheDocument();

		await page.getByRole('button', { name: 'Ver a solução de Wallace nível a nível' }).click();
		await show().click();
		await expect
			.element(page.getByTestId('wallace-rule'))
			.toHaveTextContent(/^Wallace reduz o quanto antes/);
		await expect
			.element(page.getByRole('heading', { name: 'Solução de Wallace, nível 4: CPA' }))
			.toBeVisible();
		expect(container.querySelectorAll('section.panel')).toHaveLength(6);
		expect(JSON.stringify(session.snapshot())).toBe(before);
		expect(session.helpLog()).toMatchObject({
			wallace: 1,
			wallacePath: 1,
			solution: 0,
			solutionPath: 0
		});
	});

	it('Dadda (Parhami): its own two steps and rule, applied apart from Dadda (AJU-13)', async () => {
		const { session, onapply } = await open();
		const before = JSON.stringify(session.snapshot());
		await page.getByRole('button', { name: 'Solução de Dadda (Parhami)', exact: true }).click();
		await expect.element(keepTrying()).toHaveFocus();
		await show().click();
		await expect
			.element(page.getByTestId('parhami-summary'))
			.toHaveTextContent(
				/^Solução de Dadda \(Parhami\): 4 níveis e o CPA; 13 FA, 5 HA; área 15,5 A_FA; atraso 7 T_FA/
			);
		await expect.element(page.getByTestId('solution-summary')).not.toBeInTheDocument();

		await page
			.getByRole('button', { name: 'Ver a solução de Dadda (Parhami) nível a nível' })
			.click();
		await show().click();
		await expect
			.element(page.getByTestId('parhami-rule'))
			.toHaveTextContent(/^Dadda \(Parhami\) usa as alturas-alvo de Dadda/);
		expect(JSON.stringify(session.snapshot())).toBe(before);

		await page.getByRole('button', { name: 'Usar a solução de Dadda (Parhami)' }).click();
		await replace().click();
		expect(onapply).toHaveBeenCalledWith('parhami');
		expect(session.snapshot().metrics).toMatchObject({ fa: 13, ha: 5 });
		expect(session.helpLog()).toMatchObject({
			parhami: 1,
			parhamiPath: 1,
			parhamiApplied: 1,
			solution: 0,
			solutionApplied: 0
		});
	});

	it("using Wallace's solution replaces the circuit with it (AJU-10)", async () => {
		const { session, onapply } = await open();
		await page.getByRole('button', { name: 'Solução de Wallace' }).click();
		await show().click();
		await page.getByRole('button', { name: 'Ver a solução de Wallace nível a nível' }).click();
		await show().click();
		await expect.element(useIt()).not.toBeInTheDocument();

		await page.getByRole('button', { name: 'Usar a solução de Wallace' }).click();
		await replace().click();
		expect(onapply).toHaveBeenCalledWith('wallace');
		expect(session.snapshot().metrics).toMatchObject({ fa: 13, ha: 3 });
		expect(session.helpLog()).toMatchObject({ wallaceApplied: 1, solutionApplied: 0 });
	});
});
