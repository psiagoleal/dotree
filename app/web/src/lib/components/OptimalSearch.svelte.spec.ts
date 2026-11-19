// Caminho relativo: app/web/src/lib/components/OptimalSearch.svelte.spec.ts
// MT-42: the optimum of a profile is searched off the main thread. While it runs the page
// says so and stays usable; its answer is taken once it comes, a stale one is dropped, and
// without a worker the search runs here, as before.
import { beforeAll, describe, expect, it } from 'vitest';
import { page, userEvent } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import Compression from './Compression.svelte';
import { loadCore, type Core } from '#lib/core.ts';
import { searchOptimal } from '#lib/wasm/dotree.js';

let core: Core;
beforeAll(async () => {
	core = await loadCore();
});

const PROPS = { view: 'dots' as const, outputWidth: 4, vMax: '10', widthRevealed: false };
const optimalButton = () => page.getByRole('button', { name: 'Solução ótima', exact: true });
const searching = () => page.getByTestId('optimal-searching');

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

describe('Optimal search off the main thread (MT-42)', () => {
	it('says it is searching, keeps the page usable and offers the optimum when it ends', async () => {
		const session = core.openSession('{5,5; 1,1,1,1}', {});
		const d = deferred();
		await render(Compression, { session, ...PROPS, searchOptimal: d.search });
		await expect.element(searching()).toHaveTextContent('Buscando a solução ótima…');
		await expect.element(optimalButton()).not.toBeInTheDocument();
		expect(d.requests).toHaveLength(1);
		// The page answers while the search runs: Dadda's solution is offered and opens.
		await page.getByRole('button', { name: 'Solução de Dadda', exact: true }).click();
		await expect.element(page.getByRole('dialog')).toBeVisible();
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();

		d.settle[0](true, searchOptimal(d.requests[0]));
		await expect.element(optimalButton()).toBeVisible();
		await expect.element(searching()).not.toBeInTheDocument();
		expect(session.helpLog().optimal).toBe(0);
	});

	it('a profile too large for the search says so once the worker answers', async () => {
		const session = core.openSession('{6,7,7,6; 1,1,1,1,1,1,1}', {});
		const d = deferred();
		await render(Compression, { session, ...PROPS, searchOptimal: d.search });
		d.settle[0](true, searchOptimal(d.requests[0]));
		await expect
			.element(page.getByTestId('optimal-unavailable'))
			.toHaveTextContent(/^Este perfil é grande demais/);
	});

	it('without the worker the search runs here', async () => {
		const session = core.openSession('{5,5; 1,1,1,1}', {});
		const d = deferred();
		await render(Compression, { session, ...PROPS, searchOptimal: d.search });
		d.settle[0](false);
		await expect.element(optimalButton()).toBeVisible();
	});

	it('new costs search again, and the answer for the old costs is dropped', async () => {
		const session = core.openSession('{2,2; 1,1,1}', {});
		const d = deferred();
		await render(Compression, { session, ...PROPS, searchOptimal: d.search });
		const old = searchOptimal(d.requests[0]);
		await page.getByText('Custos das células').click();
		await page.getByRole('textbox', { name: 'Área do HA', exact: true }).fill('0,3');
		await page.getByRole('textbox', { name: 'Atraso do HA', exact: true }).fill('0,3');
		await page.getByRole('button', { name: 'Aplicar custos' }).click();
		await expect.poll(() => d.requests.length).toBe(2);
		d.settle[0](true, old);
		await expect.element(searching()).toBeVisible();
		expect(session.optimalSearch()).toBe('pending');
		d.settle[1](true, searchOptimal(d.requests[1]));
		await expect.element(optimalButton()).toBeVisible();
		session.solution(false, 'optimal').free();
		expect(session.shownSolution('optimal')?.snapshot().metrics.ha).toBe(4);
	});

	it('an expression has no optimum to search', async () => {
		const session = core.openSession('A+B', { A: 2, B: 2 });
		const d = deferred();
		await render(Compression, { session, ...PROPS, searchOptimal: d.search });
		await expect.element(page.getByRole('button', { name: 'Solução de Wallace' })).toBeVisible();
		expect(d.requests).toHaveLength(0);
		await expect.element(searching()).not.toBeInTheDocument();
	});
});
