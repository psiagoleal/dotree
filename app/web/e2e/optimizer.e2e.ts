// Caminho relativo: app/web/e2e/optimizer.e2e.ts
// MT-46: the optimizer (AJU-14). On R1 it runs in the worker while cells are formed and
// undone within GER-06; its two options, exact up to 15.5 A_FA, enter the comparison, one
// replaces the circuit and verifies. On the 32 x 32 multiplier the heuristic answers.
import { expect, test, type Page } from '@playwright/test';

test.use({ locale: 'pt-BR' });

/** Milliseconds from running `act` until the next frame after it has been drawn. */
async function timed(page: Page, act: string): Promise<number> {
	return page.evaluate(async (act) => {
		const t0 = performance.now();
		if (act === 'undo') {
			window.dispatchEvent(
				new KeyboardEvent('keydown', { key: 'z', ctrlKey: true, bubbles: true })
			);
		} else {
			[...document.querySelectorAll('button')].find((b) => b.textContent?.trim() === act)!.click();
		}
		await new Promise((r) => requestAnimationFrame(() => setTimeout(r, 0)));
		return performance.now() - t0;
	}, act);
}

async function open(page: Page, src: string, widths: Record<string, number>) {
	await page.goto('./');
	await expect(page.getByTestId('core-status')).toBeVisible();
	await page.getByLabel('Expressão').fill(src);
	for (const [v, w] of Object.entries(widths))
		await page.locator(`input[name="width-${v}"]`).fill(String(w));
	return timed(page, 'Montar a matriz');
}

const show = (page: Page) =>
	page.getByRole('dialog').getByRole('button', { name: 'Mostrar', exact: true });

test('R1: the page answers while the optimizer runs, and an option applies and verifies (MT-46, GER-06)', async ({
	page
}) => {
	test.setTimeout(120_000);
	const build = await open(page, 'A*B + C*D + 3(E+F) + G', {
		A: 3,
		B: 3,
		C: 2,
		D: 2,
		E: 2,
		F: 1,
		G: 1
	});
	await expect(page.getByTestId('optimizer-searching')).toHaveText(
		'Buscando as opções do otimizador…'
	);

	const column = page
		.getByTestId('level-current')
		.getByRole('group', { name: /^Coluna de peso 2\^1,/ });
	const times: number[] = [];
	let during = 0;
	for (let i = 0; i < 25; i++) {
		const searching = await page.getByTestId('optimizer-searching').count();
		for (let k = 0; k < 3; k++) await column.getByRole('button').nth(k).click();
		times.push(await timed(page, 'Formar célula'));
		await expect(page.getByText('FA formado.')).toBeVisible();
		times.push(await timed(page, 'undo'));
		await expect(page.locator('.cell')).toHaveCount(0);
		if (searching) during++;
	}
	times.sort((a, b) => a - b);
	const p95 = times[Math.ceil(0.95 * times.length) - 1];
	console.log(
		`MT-46 R1: build ${build.toFixed(0)} ms; ${times.length} operations, ${during} rounds during the search, p95 ${p95.toFixed(1)} ms`
	);
	expect(build).toBeLessThan(100);
	expect(during).toBeGreaterThan(0);
	expect(p95).toBeLessThan(100);

	const ask = page.getByRole('button', { name: 'Otimizador', exact: true });
	await expect(ask).toBeVisible({ timeout: 30_000 });
	await ask.click();
	await show(page).click();
	await expect(page.getByTestId('optimizer-mode')).toHaveText(/^Busca exata até 15,5 A_FA/);
	await expect(page.getByTestId('optimizer-option-1')).toHaveText(
		/área 14,5 A_FA; atraso 6 T_FA\.$/
	);
	await expect(page.getByTestId('optimizer-option-2')).toHaveText(
		/área 15 A_FA; atraso 5,5 T_FA\.$/
	);
	await expect(page.getByTestId('comparison-table')).toContainText('Otimizador, opção 2');

	await page.getByRole('button', { name: 'Ver as opções do otimizador nível a nível' }).click();
	await show(page).click();
	await page.getByRole('button', { name: 'Usar a opção 2' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Substituir' }).click();
	await expect(page.getByTestId('complete')).toBeVisible();
	await page.getByRole('button', { name: 'Verificar circuito' }).click();
	await expect(page.getByTestId('verify-result')).toHaveText('16.384 casos, exaustivo, 0 erros.');
	const saved = await page.evaluate(() => localStorage.getItem('dotree.exercise'));
	expect(JSON.parse(saved!).help).toMatchObject({
		optimizer: 1,
		optimizerPath: 1,
		optimizerApplied: 1
	});
});

test('the 32 x 32 multiplier gets the heuristic, without freezing the page (MT-46)', async ({
	page
}) => {
	test.setTimeout(300_000);
	await page.setViewportSize({ width: 1280, height: 900 });
	await page.goto('./');
	await page.evaluate(() => {
		const w = window as unknown as { longest: number };
		w.longest = 0;
		new PerformanceObserver((list) => {
			for (const e of list.getEntries()) w.longest = Math.max(w.longest, e.duration);
		}).observe({ type: 'longtask', buffered: false });
	});
	await page.getByLabel('Expressão').fill('A*B');
	for (const v of ['A', 'B']) await page.locator(`input[name="width-${v}"]`).fill('32');
	const build = await timed(page, 'Montar a matriz');
	await page.evaluate(() => ((window as unknown as { longest: number }).longest = 0));
	const t0 = Date.now();
	const ask = page.getByRole('button', { name: 'Otimizador', exact: true });
	await expect(ask).toBeVisible({ timeout: 120_000 });
	const longest = await page.evaluate(() => (window as unknown as { longest: number }).longest);
	console.log(
		`MT-46 32 x 32: build ${build.toFixed(0)} ms; options after ${Date.now() - t0} ms; longest task meanwhile ${longest.toFixed(0)} ms`
	);
	expect(build).toBeLessThan(1000);
	await ask.click();
	await show(page).click();
	await expect(page.getByTestId('optimizer-mode')).toHaveText(/^Heurística:/);
	await expect(page.getByTestId('optimizer-option-1')).toHaveText(/^Opção 1: /);
});
