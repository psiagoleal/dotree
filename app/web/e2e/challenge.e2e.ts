// Caminho relativo: app/web/e2e/challenge.e2e.ts
import { expect, test, type Page } from '@playwright/test';

test.use({ locale: 'pt-BR' });

async function loadR1(page: Page) {
	await page.goto('./');
	await page.getByRole('button', { name: 'Soma de produtos' }).click();
	await expect(page.getByTestId('dot-matrix').locator('.dot')).toHaveCount(20);
}

async function answer(page: Page, value: string) {
	await page.getByLabel('Bits da saída').fill(value);
	await page.getByRole('button', { name: 'Responder' }).click();
}

const column = (page: Page, j: number) =>
	page.getByTestId('dot-matrix').getByRole('group', { name: new RegExp(`2\\^${j},`) });

test('W stays hidden until the challenge is answered; then the matrix shows all W columns (EXP-09, EXP-10, GER-01)', async ({
	page
}) => {
	await loadR1(page);
	await expect(page.locator('main')).not.toContainText('W =');
	await expect(page.locator('main')).not.toContainText('71');
	await expect(column(page, 5)).toHaveCount(0);

	await answer(page, '7');
	await expect(page.getByTestId('width-result')).toHaveText('Acertou! A saída tem W = 7 bits.');
	await expect(column(page, 6)).toHaveAccessibleName('Coluna de peso 2^6, 0 pontos');
	await expect(column(page, 7)).toHaveCount(0);
	await expect(page.getByTestId('width-path')).toHaveCount(0);
});

test('a wrong answer is told apart and W is still revealed (EXP-10)', async ({ page }) => {
	await loadR1(page);
	await answer(page, '5');
	await expect(page.getByTestId('width-result')).toHaveText(
		'Não foi dessa vez: você respondeu 5. A saída tem W = 7 bits.'
	);
});

test('the working is step 2, asked and confirmed on its own (GER-03, AJU-02)', async ({ page }) => {
	await loadR1(page);
	await page.getByRole('button', { name: 'Mostrar a resposta' }).click();
	const keepTrying = page.getByRole('button', { name: 'Vou tentar mais um pouco' });
	await expect(keepTrying).toBeFocused();
	await keepTrying.click();
	await expect(page.getByTestId('width-result')).toHaveCount(0);

	await page.getByRole('button', { name: 'Mostrar a resposta' }).click();
	await page.getByRole('button', { name: 'Mostrar', exact: true }).click();
	await expect(page.getByTestId('width-result')).toHaveText('A saída tem W = 7 bits.');
	await expect(page.getByTestId('width-path')).toHaveCount(0);

	await page.getByRole('button', { name: 'Mostrar a conta' }).click();
	await expect(keepTrying).toBeFocused();
	await page.keyboard.press('Escape');
	await expect(page.getByTestId('width-path')).toHaveCount(0);

	await page.getByRole('button', { name: 'Mostrar a conta' }).click();
	await page.getByRole('button', { name: 'Mostrar', exact: true }).click();
	await expect(page.getByTestId('width-path')).toContainText('7·7 + 3·3 + 3·3 + 3·1 + 1');
	await expect(page.getByTestId('width-path')).toContainText('64 ≤ 71 < 128');
});

test('a new matrix starts a new challenge', async ({ page }) => {
	await loadR1(page);
	await answer(page, '7');
	await page.getByRole('button', { name: 'Multiplicador 4×4' }).click();
	await expect(page.getByTestId('width-result')).toHaveCount(0);
	await expect(page.getByLabel('Bits da saída')).toBeVisible();
	await expect(column(page, 7)).toHaveCount(0); // 4x4: dots up to 2^6, W = 8 still hidden
});

test('V toggles dots and indexed variables, outside text fields, and is remembered (VIS-01, VIS-03)', async ({
	page
}) => {
	await loadR1(page);
	const dot = page.getByRole('img', { name: 'a1_b0' });
	await expect(dot).toHaveText('');

	await page.locator('main h2').click(); // focus outside any text field
	await page.keyboard.press('v');
	await expect(dot).toHaveText('a1b0');
	await expect(page.getByRole('img', { name: 'e0' }).first()).toHaveText('e0');
	await expect(page.getByRole('button', { name: 'Variáveis indexadas' })).toHaveAttribute(
		'aria-pressed',
		'true'
	);

	// Typing V in the expression is a variable, not the shortcut.
	await page.getByLabel('Expressão').press('V');
	await expect(dot).toHaveText('a1b0');

	await page.reload();
	await loadR1(page);
	await expect(page.getByRole('img', { name: 'a1_b0' })).toHaveText('a1b0');

	await page.getByRole('button', { name: 'Pontos' }).click();
	await expect(page.getByRole('img', { name: 'a1_b0' })).toHaveText('');
});
