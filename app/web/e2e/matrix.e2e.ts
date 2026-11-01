// Caminho relativo: app/web/e2e/matrix.e2e.ts
import { expect, test } from '@playwright/test';

test.use({ locale: 'pt-BR' });

test('example R1 builds the dot diagram of spec 00 (EXP-01, EXP-12)', async ({ page }) => {
	await page.goto('./');
	await page.getByRole('button', { name: 'Soma de produtos' }).click();
	await expect(page.getByTestId('matrix-summary')).toHaveText('20 pontos · 13 portas AND');
	const matrix = page.getByTestId('dot-matrix');
	await expect(matrix.locator('.dot')).toHaveCount(20);
	// Heights 1, 2, 2, 7, 5 from 2^4 down to 2^0 (EXP-09: no 2^5 or 2^6 shown).
	for (const [col, h] of [
		[4, 1],
		[3, 2],
		[2, 5],
		[1, 7],
		[0, 5]
	] as const) {
		await expect(
			matrix.getByRole('group', { name: `Coluna de peso 2^${col}, ${h} pontos` })
		).toBeVisible();
	}
	await expect(matrix.getByRole('group', { name: /2\^5/ })).toHaveCount(0);
	await expect(page.getByTestId('term-legend').locator('li')).toHaveCount(5);
});

test('typing an expression and its widths builds the matrix', async ({ page }) => {
	await page.goto('./');
	await page.getByLabel('Expressão').fill('A*B');
	await page.locator('input[name="width-A"]').fill('2');
	await page.locator('input[name="width-B"]').fill('2');
	await page.getByRole('button', { name: 'Montar a matriz' }).click();
	await expect(page.getByTestId('matrix-summary')).toHaveText('4 pontos · 4 portas AND');
});

test('a syntax error names the column and what was expected (EXP-05)', async ({ page }) => {
	await page.goto('./');
	await page.getByLabel('Expressão').fill('A+');
	await page.getByRole('button', { name: 'Montar a matriz' }).click();
	await expect(page.getByTestId('build-error')).toHaveText(
		'A expressão terminou antes da hora (coluna 3). Esperado: uma variável, um número ou “(”.'
	);
	await expect(page.getByTestId('dot-matrix')).toHaveCount(0);
});

test('a dot shows its signal on keyboard focus (VIS-04)', async ({ page }) => {
	await page.goto('./');
	await page.getByRole('button', { name: 'Multiplicador 4×4' }).click();
	const dot = page.getByRole('img', { name: 'a1_b2' });
	await dot.focus();
	await expect(dot).toBeFocused();
});
