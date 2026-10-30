// Caminho relativo: app/web/e2e/app.e2e.ts
import { expect, test } from '@playwright/test';

test('loads under the Pages base path and runs the WASM core (ENT-03)', async ({ page }) => {
	await page.goto('./');
	await expect(page.getByRole('heading', { level: 1, name: 'dotree' })).toBeVisible();
	await expect(page.getByTestId('core-status')).toContainText(/\d+\.\d+\.\d+/);
});

test('first visit follows the browser language (IDI-03)', async ({ browser }) => {
	for (const [locale, tagline] of [
		['pt-BR', /Monte a matriz de pontos/],
		['pt-PT', /Monte a matriz de pontos/],
		['en-US', /Build the dot diagram/]
	] as const) {
		const context = await browser.newContext({ locale });
		const page = await context.newPage();
		await page.goto('./');
		await expect(page.getByText(tagline)).toBeVisible();
		await context.close();
	}
});

test('language choice is remembered across visits (IDI-04)', async ({ browser }) => {
	const context = await browser.newContext({ locale: 'pt-BR' });
	const page = await context.newPage();
	await page.goto('./');
	await page.getByRole('button', { name: 'English' }).click();
	await expect(page.getByText(/Build the dot diagram/)).toBeVisible();
	await page.reload();
	await expect(page.getByText(/Build the dot diagram/)).toBeVisible();
	await context.close();
});

test('a broken WASM load shows an explanation (GER-09)', async ({ page }) => {
	await page.route('**/*.wasm', (route) => route.abort());
	await page.goto('./');
	await expect(page.getByTestId('core-error')).toBeVisible();
	await expect(page.getByRole('link', { name: /issue/i })).toBeVisible();
});
