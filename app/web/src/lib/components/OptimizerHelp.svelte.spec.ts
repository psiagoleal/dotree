// Caminho relativo: app/web/src/lib/components/OptimizerHelp.svelte.spec.ts
// MT-46: the optimizer (AJU-14). Searched off the main thread when the exercise opens and
// after new costs; revealed in two confirmed, counted steps; its options enter the
// comparison and each can replace the circuit as one undo step.
import { beforeAll, describe, expect, it } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import Compression from './Compression.svelte';
import { loadCore, type Core } from '#lib/core.ts';
import { searchOptimizer } from '#lib/wasm/dotree.js';

let core: Core;
beforeAll(async () => {
	core = await loadCore();
});

const PROPS = { view: 'dots' as const, outputWidth: 7, vMax: '71', widthRevealed: false };
const R1 = ['A*B + C*D + 3(E+F) + G', { A: 3, B: 3, C: 2, D: 2, E: 2, F: 1, G: 1 }] as const;
const ask = () => page.getByRole('button', { name: 'Otimizador', exact: true });
const askPath = () =>
	page.getByRole('button', { name: 'Ver as opções do otimizador nível a nível' });
const show = () => page.getByRole('dialog').getByRole('button', { name: 'Mostrar' });
const searching = () => page.getByTestId('optimizer-searching');

/** A search the test settles by hand; `requests` records what the page sent. */
function deferred() {
	const requests: string[] = [];
	const settle: ((ok: boolean, result?: string) => void)[] = [];
	const search = (request: string) => {
		requests.push(request);
		return new Promise<string>((resolve, reject) =>
			settle.push((ok, result) => (ok ? resolve(result!) : reject(new Error('gone'))))
		);
	};
	return { requests, settle, search };
}

async function opened(session = core.openSession(...R1)) {
	const d = deferred();
	await render(Compression, { session, ...PROPS, searchOptimizer: d.search });
	return { session, d };
}

describe('Optimizer (AJU-14, MT-46)', () => {
	it('searches in the worker, then shows the options only when asked, counted', async () => {
		const { session, d } = await opened();
		await expect.element(searching()).toHaveTextContent('Buscando as opções do otimizador…');
		await expect.element(ask()).not.toBeInTheDocument();
		d.settle[0](true, searchOptimizer(d.requests[0]));
		await expect.element(ask()).toBeVisible();
		await expect.element(page.getByTestId('optimizer-options')).not.toBeInTheDocument();
		await expect.element(page.getByTestId('comparison-empty')).toBeVisible();
		expect(session.helpLog().optimizer).toBe(0);

		await ask().click();
		await show().click();
		await expect
			.element(page.getByTestId('optimizer-mode'))
			.toHaveTextContent(/^Busca exata até 15,5 A_FA/);
		await expect
			.element(page.getByTestId('optimizer-option-1'))
			.toHaveTextContent(
				/^Opção 1: \d níveis e o CPA; 1\d FA, \d HA; área 14,5 A_FA; atraso 6 T_FA\.$/
			);
		await expect
			.element(page.getByTestId('optimizer-option-2'))
			.toHaveTextContent(/área 15 A_FA; atraso 5,5 T_FA\.$/);
		expect(session.helpLog().optimizer).toBe(1);
		// Both options enter the comparison, uncounted.
		const head = page.getByTestId('comparison-table').getByRole('columnheader');
		await expect.element(head.nth(2)).toHaveTextContent('Otimizador, opção 1');
		await expect.element(head.nth(3)).toHaveTextContent('Otimizador, opção 2');
		await expect.element(page.getByTestId('optimizer')).not.toBeInTheDocument();
		expect(session.helpLog().optimizer).toBe(1);
	});

	it('a dialog open when the search ends stays open', async () => {
		const { d } = await opened();
		await page.getByRole('button', { name: 'Solução de Dadda', exact: true }).click();
		await expect.element(page.getByRole('dialog')).toBeVisible();
		d.settle[0](true, searchOptimizer(d.requests[0]));
		await expect.element(ask()).toBeVisible();
		await show().click();
		await expect.element(page.getByTestId('solution-summary')).toBeVisible();
	});

	it('the levels come with step 2, and an option replaces the circuit as one undo step', async () => {
		const { session, d } = await opened();
		d.settle[0](true, searchOptimizer(d.requests[0]));
		await ask().click();
		await show().click();
		await askPath().click();
		await show().click();
		await expect.element(page.getByTestId('optimizer-rule')).toBeVisible();
		await expect.element(page.getByTestId('optimizer-1-level-0')).toBeVisible();
		await expect.element(page.getByTestId('optimizer-2-result')).toBeVisible();
		await expect.element(page.getByTestId('height-table-optimizer-2')).toBeVisible();
		expect(session.helpLog().optimizerPath).toBe(1);

		await page.getByRole('button', { name: 'Usar a opção 2' }).click();
		await expect
			.element(page.getByRole('dialog'))
			.toHaveTextContent(/Trocar o seu circuito pela opção 2 do otimizador\?/);
		await page.getByRole('dialog').getByRole('button', { name: 'Substituir' }).click();
		await expect.element(page.getByTestId('complete')).toBeVisible();
		const snap = session.snapshot();
		expect([snap.metrics.area, snap.metrics.delay]).toEqual([1500, 550]);
		expect(session.helpLog().optimizerApplied).toBe(1);
		expect(session.undo()).toBe(true);
		expect(session.snapshot().metrics.area).toBe(0);
	});

	it('new costs search again, and the answer for the old costs is dropped', async () => {
		const { session, d } = await opened();
		const old = searchOptimizer(d.requests[0]);
		await page.getByText('Custos das células').click();
		await page.getByRole('textbox', { name: 'Área do HA', exact: true }).fill('0,3');
		await page.getByRole('button', { name: 'Aplicar custos' }).click();
		await expect.poll(() => d.requests.length).toBe(2);
		d.settle[0](true, old);
		await expect.element(searching()).toBeVisible();
		expect(session.optimizerSearch()).toBe('pending');
		d.settle[1](true, searchOptimizer(d.requests[1]));
		await expect.element(ask()).toBeVisible();
	});

	it('without the worker the optimizer runs here', async () => {
		const { d } = await opened();
		d.settle[0](false);
		await expect.element(ask()).toBeVisible();
	});

	it('a profile has options too, at the profile', async () => {
		const { d } = await opened(core.openSession('{5,5; 1,1,1,1}', {}));
		d.settle[0](true, searchOptimizer(d.requests[0]));
		await ask().click();
		await show().click();
		await expect.element(page.getByTestId('optimizer-mode')).toHaveTextContent(/^Busca exata:/);
		await expect
			.element(page.getByTestId('optimizer-option-1'))
			.toHaveTextContent(/área 6 A_FA; atraso 4 T_FA\.$/);
	});

	it('a reopened exercise shows what was revealed, uncounted', async () => {
		const first = core.openSession(...R1);
		first.searchOptimizer();
		first.optimizer(false);
		first.optimizer(true);
		const restored = core.restoreSession(first.exercise());
		first.free();
		if (!restored.ok) throw new Error('the exercise opens');
		const { session, d } = await opened(restored.session);
		d.settle[0](true, searchOptimizer(d.requests[0]));
		await expect.element(page.getByTestId('optimizer-2-result')).toBeVisible();
		await expect.element(askPath()).not.toBeInTheDocument();
		expect(session.helpLog()).toMatchObject({ optimizer: 1, optimizerPath: 1 });
	});
});
