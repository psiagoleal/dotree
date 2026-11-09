// Caminho relativo: app/web/e2e/quality.e2e.ts
// MT-16: non-functional requirements of spec 00 — everything runs in the browser with the
// network gone (GER-05), forming and undoing stay under 100 ms (GER-06), no serious
// accessibility violation (GER-07), and every request goes to the site itself (GER-08).
import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';
import { r2ByMouse } from './r2';

test.use({ locale: 'pt-BR' });

/** R2 complete, verified, with diagram and VHDL on screen. */
async function r2Everything(page: Page) {
	await r2ByMouse(page);
	await page.getByRole('button', { name: 'Fechar com CPA' }).click();
	await page.getByRole('button', { name: 'Verificar circuito' }).click();
	await expect(page.getByTestId('verify-result')).toHaveText('16.384 casos, exaustivo, 0 erros.');
	const zip = page.waitForEvent('download');
	await page.getByRole('button', { name: 'Baixar todos (.zip)' }).click();
	await zip;
}

test('with the network gone after loading, R2 still goes to the end (GER-05)', async ({
	page,
	context
}) => {
	await page.goto('./');
	await expect(page.getByTestId('core-status')).toBeVisible();
	await context.setOffline(true);
	await r2Everything(page);
	await expect(page.getByTestId('metrics')).toContainText(/FA\s*13/);
});

test('every request goes to the site itself (GER-08)', async ({ page, baseURL }) => {
	const origin = new URL(baseURL!).origin;
	const elsewhere: string[] = [];
	page.on('request', (r) => {
		const url = new URL(r.url());
		if (url.protocol !== 'blob:' && url.protocol !== 'data:' && url.origin !== origin) {
			elsewhere.push(r.url());
		}
	});
	await page.goto('./');
	await r2Everything(page);
	await page.reload();
	await page.getByRole('button', { name: 'Retomar' }).click();
	expect(elsewhere).toEqual([]);
});

/** Milliseconds from a click (or key) until the next frame after it has been drawn. */
async function timed(page: Page, act: 'form' | 'undo'): Promise<number> {
	return page.evaluate(async (act) => {
		const t0 = performance.now();
		if (act === 'form') {
			[...document.querySelectorAll('button')]
				.find((b) => b.textContent?.trim() === 'Formar célula')!
				.click();
		} else {
			window.dispatchEvent(
				new KeyboardEvent('keydown', { key: 'z', ctrlKey: true, bubbles: true })
			);
		}
		await new Promise((r) => requestAnimationFrame(() => setTimeout(r, 0)));
		return performance.now() - t0;
	}, act);
}

test('forming and undoing on a 16 x 16 matrix stay under 100 ms at the 95th percentile (GER-06)', async ({
	page
}) => {
	test.setTimeout(120_000);
	await page.goto('./');
	await page.getByLabel('Expressão').fill('A*B');
	for (const v of ['A', 'B']) await page.locator(`input[name="width-${v}"]`).fill('16');
	await page.getByRole('button', { name: 'Montar a matriz' }).click();
	await expect(page.getByTestId('matrix-summary')).toContainText('256');

	const column = page
		.getByTestId('level-current')
		.getByRole('group', { name: /^Coluna de peso 2\^15,/ });
	const times: number[] = [];
	for (let i = 0; i < 25; i++) {
		for (let k = 0; k < 3; k++) await column.getByRole('button').nth(k).click();
		times.push(await timed(page, 'form'));
		await expect(page.getByText('FA formado.')).toBeVisible();
		times.push(await timed(page, 'undo'));
		await expect(page.locator('.cell')).toHaveCount(0);
	}
	times.sort((a, b) => a - b);
	const p95 = times[Math.ceil(0.95 * times.length) - 1];
	console.log(
		`GER-06: ${times.length} operations, median ${times[25].toFixed(1)} ms, p95 ${p95.toFixed(1)} ms`
	);
	expect(p95).toBeLessThan(100);
});

async function audit(page: Page, label: string) {
	const { violations } = await new AxeBuilder({ page }).analyze();
	const serious = violations
		.filter((v) => v.impact === 'serious' || v.impact === 'critical')
		.map(
			(v) => `${label}: ${v.id} (${v.impact}) ${v.nodes.map((n) => n.target.join(' ')).join(' | ')}`
		);
	expect(serious).toEqual([]);
}

for (const scheme of ['light', 'dark'] as const) {
	test(`no serious or critical accessibility violation, ${scheme} theme (GER-07)`, async ({
		page
	}) => {
		test.setTimeout(120_000);
		await page.emulateMedia({ colorScheme: scheme });
		await page.goto('./');
		await expect(page.getByTestId('core-status')).toBeVisible();
		await audit(page, 'start');

		await page.getByRole('button', { name: 'Soma de produtos' }).click();
		await page.getByRole('button', { name: 'Mostrar a resposta' }).click();
		await audit(page, 'dialog');
		await page.getByRole('dialog').getByRole('button', { name: 'Mostrar', exact: true }).click();
		await page.keyboard.press('v');
		await audit(page, 'matrix, indexed view');
		await page.keyboard.press('v');

		await page.reload();
		await audit(page, 'saved offer');
		await page.getByRole('button', { name: 'Descartar' }).click();
		await page.getByRole('dialog').getByRole('button', { name: 'Descartar' }).click();

		await r2Everything(page);
		for (const method of ['Dadda', 'Wallace']) {
			await page.getByRole('button', { name: `Solução de ${method}`, exact: true }).click();
			await page.getByRole('dialog').getByRole('button', { name: 'Mostrar', exact: true }).click();
		}
		await audit(page, 'complete circuit');
	});
}
