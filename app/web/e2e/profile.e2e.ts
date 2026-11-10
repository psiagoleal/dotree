// Caminho relativo: app/web/e2e/profile.e2e.ts
// MT-31b: input by column heights (EXP-16, EXP-17), the output profile (CMP-19) and its
// optimal circuit and block name as counted helps (AJU-12).
import { expect, test, type Page } from '@playwright/test';
import { formByMouse } from './r2';

test.use({ locale: 'pt-BR' });

const show = (page: Page) =>
	page.getByRole('dialog').getByRole('button', { name: 'Mostrar', exact: true });

async function build(page: Page, text: string) {
	await page.getByLabel('Expressão').fill(text);
	await page.getByRole('button', { name: 'Montar a matriz' }).click();
}

test('a profile builds without widths and shows what it asks, revealing nothing (EXP-16, GER-01)', async ({
	page
}) => {
	await page.goto('./');
	await page.getByRole('button', { name: 'Perfil de alturas' }).click();
	await expect(page.getByLabel('Expressão')).toHaveValue('{5,5; 1,1,1,1}');
	await expect(page.locator('fieldset')).toHaveCount(0);
	await expect(page.getByTestId('matrix-summary')).toContainText('10 pontos');
	await expect(page.getByTestId('profile')).toContainText('Pedido: {1, 1, 1, 1}');
	await expect(page.getByTestId('profile-over')).toContainText('a coluna 2^1 tem 5');
	for (const id of ['profile-block', 'optimal-summary', 'profile-reached']) {
		await expect(page.getByTestId(id)).toHaveCount(0);
	}
	const text = await page.locator('main').innerText();
	expect(text).not.toMatch(/Solução ótima:|É um |São \d+ FA/);
});

test('a malformed or impossible profile is refused with the column (EXP-17)', async ({ page }) => {
	await page.goto('./');
	await build(page, '{2,2 1}');
	await expect(page.getByTestId('build-error')).toHaveText(
		/^Perfil de alturas malformado na coluna 6\./
	);
	await build(page, '{2; 2,0}');
	await expect(page.getByTestId('build-error')).toHaveText(/^A saída pede 0 pontos na coluna 2\^0/);
});

test('reaching the profile shows area and delay and closes with the help log (CMP-19)', async ({
	page
}) => {
	await page.goto('./');
	await build(page, '{3; 1,1}');
	await formByMouse(page, 0, ['a0', 'b0', 'c0']);
	// Formed, not concluded: column 0 still holds its three dots.
	await expect(page.getByTestId('profile-over')).toHaveText(
		'Ainda não: a coluna 2^0 tem 3, o perfil pede 1.'
	);
	await page.getByRole('button', { name: 'Concluir nível' }).click();
	await expect(page.getByTestId('profile-reached')).toHaveText(
		'Perfil atingido: área 1 A_FA, atraso 1 T_FA.'
	);
	await expect(page.getByTestId('help-log')).toHaveText('Ajudas usadas: nenhuma.');
});

test('a redundant output: the optimum and the block, counted, kept after a reload (AJU-12)', async ({
	page
}) => {
	await page.goto('./');
	await build(page, '{5; 2,1}');

	await page.getByRole('button', { name: 'Qual bloco é este?' }).click();
	await show(page).click();
	await expect(page.getByTestId('profile-block')).toHaveText(/^É um compressor/);

	await page.getByRole('button', { name: 'Solução ótima', exact: true }).click();
	await show(page).click();
	// k) of the reference set: 2 A_FA and 2 T_FA.
	await expect(page.getByTestId('optimal-summary')).toHaveText(
		'Solução ótima: 2 níveis; 2 FA, 0 HA; área 2 A_FA; atraso 2 T_FA.'
	);
	await page.getByRole('button', { name: 'Ver a solução ótima nível a nível' }).click();
	await show(page).click();
	await page.getByRole('button', { name: 'Usar a solução ótima' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Substituir' }).click();

	await expect(page.getByTestId('profile-reached')).toHaveText(
		'Perfil atingido: área 2 A_FA, atraso 2 T_FA.'
	);
	await expect(page.getByTestId('help-log')).toHaveText(
		'Ajudas usadas: solução ótima: 1 (nível a nível: 1) · solução ótima aplicada: 1 vez · nome do bloco: 1'
	);
	// Not complete: column 1 keeps two bits, so there is no result to verify.
	await expect(page.getByTestId('complete')).toHaveCount(0);

	await page.reload();
	await page.getByRole('button', { name: 'Retomar' }).click();
	await expect(page.getByLabel('Expressão')).toHaveValue('{5; 2,1}');
	await expect(page.getByTestId('profile-block')).toHaveText(/^É um compressor/);
	await expect(page.getByTestId('profile-reached')).toBeVisible();
	await expect(page.getByTestId('optimal-summary')).toBeVisible();
});
