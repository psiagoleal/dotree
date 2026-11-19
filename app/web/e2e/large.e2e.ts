// Caminho relativo: app/web/e2e/large.e2e.ts
// MT-25a: the 32 x 32 multiplier at the limits of spec 01 (1024 dots, 64 input bits). The
// matrix is navigated with the zoom and sideways scrolling (VIS-05), Dadda's solution is
// applied and verified by sample, and the diagram and the wiring table stay usable (DIA-08).
// The times of applying, undoing and redoing the solution are printed (GER-06 note).
import { expect, test, type Locator, type Page } from '@playwright/test';

test.use({ locale: 'pt-BR', viewport: { width: 1280, height: 900 } });

/** Milliseconds from running `act` until the next frame after it has been drawn. */
async function timed(page: Page, act: string): Promise<number> {
	return page.evaluate(async (act) => {
		const t0 = performance.now();
		if (act === 'undo' || act === 'redo') {
			const key = act === 'undo' ? 'z' : 'y';
			window.dispatchEvent(new KeyboardEvent('keydown', { key, ctrlKey: true, bubbles: true }));
		} else {
			[...document.querySelectorAll('button')].find((b) => b.textContent?.trim() === act)!.click();
		}
		await new Promise((r) => requestAnimationFrame(() => setTimeout(r, 0)));
		return performance.now() - t0;
	}, act);
}

/** True when `inner` lies sideways within what `box` shows. */
async function inside(inner: Locator, box: Locator): Promise<boolean> {
	const [a, b] = [await inner.boundingBox(), await box.boundingBox()];
	return a !== null && b !== null && a.x >= b.x - 1 && a.x + a.width <= b.x + b.width + 1;
}

test('the 32 x 32 multiplier: navigate, apply Dadda, verify, diagram and table (MT-25a)', async ({
	page
}) => {
	test.setTimeout(300_000);
	await page.goto('./');
	await page.getByLabel('Expressão').fill('A*B');
	for (const v of ['A', 'B']) await page.locator(`input[name="width-${v}"]`).fill('32');
	await page.getByRole('button', { name: 'Montar a matriz' }).click();
	await expect(page.getByTestId('matrix-summary')).toHaveText('1024 pontos · 1024 portas AND');

	// VIS-05: at 100% the low columns are off to the right; fitting brings all 64 in view,
	// and the points stay buttons.
	const level = page.getByTestId('level-current');
	const box = level.locator('[data-zoom-area]');
	const low = level.getByRole('group', { name: /^Coluna de peso 2\^0,/ });
	const high = level.getByRole('group', { name: /^Coluna de peso 2\^62,/ });
	expect(await inside(low, box)).toBe(false);
	const zoom = page.getByRole('group', { name: 'Zoom da matriz' });
	await zoom.getByRole('button', { name: 'Ajustar à largura' }).click();
	await expect(zoom.locator('output')).toHaveText('33%');
	expect(await inside(low, box)).toBe(true);
	expect(await inside(high, box)).toBe(true);
	await low.getByRole('button').click();
	await expect(low.getByRole('button')).toHaveAttribute('aria-pressed', 'true');
	await low.getByRole('button').click();
	// Fitting again from another zoom lands on the same 33%.
	await zoom.getByRole('button', { name: 'Aumentar' }).click();
	await expect(zoom.locator('output')).toHaveText('50%');
	await zoom.getByRole('button', { name: 'Ajustar à largura' }).click();
	await expect(zoom.locator('output')).toHaveText('33%');

	// Back to 100%: the column is reached by scrolling the level sideways.
	for (const step of ['50%', '75%', '100%']) {
		await zoom.getByRole('button', { name: 'Aumentar' }).click();
		await expect(zoom.locator('output')).toHaveText(step);
	}
	expect(await inside(low, box)).toBe(false);
	await low.getByRole('button').focus();
	expect(await inside(low, box)).toBe(true);

	const show = page.getByRole('dialog').getByRole('button', { name: 'Mostrar', exact: true });
	await page.getByRole('button', { name: 'Solução de Dadda', exact: true }).click();
	await show.click();
	await page.getByRole('button', { name: 'Ver a solução de Dadda nível a nível' }).click();
	await show.click();
	await page.getByRole('button', { name: 'Usar a solução de Dadda', exact: true }).click();
	const apply = await timed(page, 'Substituir');
	await expect(page.getByTestId('complete')).toBeVisible();
	await expect(page.getByTestId('metrics')).toContainText(/FA\s*\d+/);
	const undo = await timed(page, 'undo');
	await expect(page.getByTestId('complete')).toHaveCount(0);
	const redo = await timed(page, 'redo');
	await expect(page.getByTestId('complete')).toBeVisible();
	console.log(
		`MT-25a Dadda 32 x 32: apply ${apply.toFixed(0)} ms, undo ${undo.toFixed(0)} ms, redo ${redo.toFixed(0)} ms`
	);

	await page.getByRole('button', { name: 'Verificar circuito' }).click();
	await expect(page.getByTestId('verify-result')).toHaveText(
		'1.048.576 casos, por amostra, 0 erros.',
		{ timeout: 120_000 }
	);

	// DIA-08: the diagram zooms down to 25% and still scrolls; the wiring table scrolls in
	// its own box with the header in view.
	const diagram = page.getByTestId('diagram');
	const dzoom = page.getByRole('group', { name: 'Zoom do diagrama' });
	await dzoom.getByRole('button', { name: 'Ajustar à largura' }).click();
	await expect(dzoom.locator('output')).toHaveText('25%');
	await expect(dzoom.getByRole('button', { name: 'Diminuir' })).toBeDisabled();
	const [drawn, natural] = await diagram
		.locator('svg')
		.evaluate((s) => [s.getBoundingClientRect().width, Number(s.getAttribute('width'))]);
	expect(natural).toBeGreaterThan(20_000);
	expect(drawn / natural).toBeCloseTo(0.25, 2);
	expect(await diagram.evaluate((d) => d.scrollWidth > d.clientWidth)).toBe(true);

	const table = page.getByTestId('wiring-table');
	expect(await table.locator('tbody tr').count()).toBeGreaterThan(900);
	const wrap = page.getByRole('region', { name: 'Tabela de ligações' });
	const viewport = page.viewportSize()!.height;
	expect((await wrap.boundingBox())!.height).toBeLessThanOrEqual(0.7 * viewport + 1);
	await wrap.evaluate((w) => (w.scrollTop = 5000));
	const head = table.locator('thead th').first();
	const [h, w] = [await head.boundingBox(), await wrap.boundingBox()];
	expect(Math.abs(h!.y - w!.y)).toBeLessThan(2);
});
