// Caminho relativo: app/web/e2e/optimal.e2e.ts
// MT-42: the optimum of a height profile is searched in the worker. Opening the largest
// accepted profile (26 dots) no longer freezes the page: cells are formed and undone while
// the search runs, within GER-06, and the answer arrives afterwards.
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

test('the page answers while the optimum of the largest profile is searched (MT-42, GER-06)', async ({
	page
}) => {
	test.setTimeout(120_000);
	await page.goto('./');
	await expect(page.getByTestId('core-status')).toBeVisible();
	await page.getByLabel('Expressão').fill('{6,7,7,6; 1,1,1,1,1,1,1}');
	// Before MT-42 the search ran here and this took about 0.7 s.
	const build = await timed(page, 'Montar a matriz');
	await expect(page.getByTestId('optimal-searching')).toHaveText('Buscando a solução ótima…');

	const column = page
		.getByTestId('level-current')
		.getByRole('group', { name: /^Coluna de peso 2\^1,/ });
	const times: number[] = [];
	let during = 0;
	for (let i = 0; i < 25; i++) {
		const searching = await page.getByTestId('optimal-searching').count();
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
		`MT-42: build ${build.toFixed(0)} ms; ${times.length} operations, ${during} rounds during the search, p95 ${p95.toFixed(1)} ms`
	);
	expect(build).toBeLessThan(100);
	expect(during).toBeGreaterThan(0);
	expect(p95).toBeLessThan(100);

	await expect(page.getByTestId('optimal-unavailable')).toHaveText(/^Este perfil é grande demais/, {
		timeout: 30_000
	});
	await expect(page.getByTestId('optimal-searching')).toHaveCount(0);
	// Searching counts nothing (AJU-12).
	const saved = await page.evaluate(() => localStorage.getItem('dotree.exercise'));
	expect(JSON.parse(saved!).help.optimal).toBe(0);
});

test('a profile with an optimum offers it once the worker answers, and it applies (MT-42)', async ({
	page
}) => {
	await page.goto('./');
	await page.getByLabel('Expressão').fill('{5,5; 1,1,1,1}');
	await page.getByRole('button', { name: 'Montar a matriz' }).click();
	const show = page.getByRole('dialog').getByRole('button', { name: 'Mostrar', exact: true });
	await page.getByRole('button', { name: 'Solução ótima', exact: true }).click();
	await show.click();
	await expect(page.getByTestId('optimal-summary')).toContainText(
		'6 FA, 0 HA; área 6 A_FA; atraso 4 T_FA.'
	);
	await page.getByRole('button', { name: 'Ver a solução ótima nível a nível' }).click();
	await show.click();
	await page.getByRole('button', { name: 'Usar a solução ótima' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Substituir' }).click();
	await expect(page.getByTestId('metrics')).toContainText(/FA\s*6/);
});
