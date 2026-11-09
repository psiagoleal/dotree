// Caminho relativo: app/web/e2e/constant.e2e.ts
// MT-28: an additive constant (EXP-15), from the expression to a verified circuit and its
// VHDL. The expression is the project's own: 3(A + 1) + B = 3A + 3 + B, A and B of 2 bits.
import { expect, test } from '@playwright/test';

test.use({ locale: 'pt-BR' });

test('a constant becomes fixed dots, compresses like the others and verifies (EXP-15)', async ({
	page
}) => {
	await page.goto('./');
	await page.getByLabel('Expressão').fill('3(A + 1) + B');
	for (const v of ['A', 'B']) await page.locator(`input[name="width-${v}"]`).fill('2');
	await page.getByRole('button', { name: 'Montar a matriz' }).click();

	// One term for the constant, in its own colour, where the first constant was.
	await expect(page.getByTestId('term-legend').locator('.term')).toHaveText(['3A', '3', 'B']);
	// 3 = 1 + 2: a fixed dot in 2^0 and one in 2^1, called 1.
	const level = page.getByTestId('level-current');
	const fixed = level.locator('button[data-signal="one"]');
	await expect(fixed).toHaveCount(2);
	await expect(fixed.first()).toHaveAccessibleName('1');
	await expect(fixed.first()).toHaveAttribute('style', /--dot-term-2/);

	// Dadda's solution, applied (AJU-09), compresses the fixed dots too.
	const show = page.getByRole('dialog').getByRole('button', { name: 'Mostrar', exact: true });
	await page.getByRole('button', { name: 'Solução automática', exact: true }).click();
	await show.click();
	await page.getByRole('button', { name: 'Ver a solução nível a nível' }).click();
	await show.click();
	await page.getByRole('button', { name: 'Usar esta solução' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Substituir' }).click();
	await expect(page.getByTestId('complete')).toBeVisible();

	await page.getByRole('button', { name: 'Verificar circuito' }).click();
	await expect(page.getByTestId('verify-result')).toHaveText('16 casos, exaustivo, 0 erros.');

	const top = page.locator('pre[data-file="dotree_circuito.vhd"]');
	await expect(top).toContainText("one <= '1';");
	await expect(top).toContainText('R = 3A + 3 + B');
});
