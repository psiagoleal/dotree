// Caminho relativo: app/web/e2e/costs.e2e.ts
// MT-45: configurable costs (GER-12). Changing them recomputes the metrics and the
// solutions, survives a reload, carries over to the next matrix and is undone only by
// "Restore the default"; nothing is counted as help.
import { expect, test, type Page } from '@playwright/test';
import { formByMouse } from './r2';

test.use({ locale: 'pt-BR' });

const metrics = (page: Page) => page.getByTestId('metrics');
const summary = (page: Page) => page.getByTestId('costs-summary');
const show = (page: Page) =>
	page.getByRole('dialog').getByRole('button', { name: 'Mostrar', exact: true });

async function build(page: Page, text: string, widths: Record<string, string> = {}) {
	await page.getByLabel('Expressão').fill(text);
	for (const [v, w] of Object.entries(widths))
		await page.locator(`input[name="width-${v}"]`).fill(w);
	await page.getByRole('button', { name: 'Montar a matriz' }).click();
}

async function setCost(page: Page, name: string, value: string) {
	await page.getByRole('textbox', { name, exact: true }).fill(value);
}

test('a faster FA moves the delay, survives a reload, carries over and is restored (GER-12)', async ({
	page
}) => {
	await page.goto('./');
	// The hand case of the core: FA 1 on column 0, HA 2 on column 1, then the CPA.
	await build(page, 'A+B+C+2(D+E)', { A: '1', B: '1', C: '1', D: '1', E: '1' });
	await formByMouse(page, 0, ['a0', 'b0', 'c0']);
	await formByMouse(page, 1, ['d0', 'e0']);
	await page.getByRole('button', { name: 'Concluir nível' }).click();
	await page.getByRole('button', { name: 'Fechar com CPA' }).click();
	await expect(metrics(page)).toContainText('Área 2,5 AFA');
	await expect(metrics(page)).toContainText('Atraso 2 TFA');

	await summary(page).click();
	await setCost(page, 'Atraso do FA', '0,4');
	await page.getByRole('button', { name: 'Aplicar custos' }).click();
	await expect(metrics(page)).toContainText('Atraso 1,5 TFA');
	await expect(metrics(page)).toContainText('Área 2,5 AFA');
	await expect(summary(page)).toHaveText('FA: área 1, atraso 0,4 · HA: área 0,5, atraso 0,5');
	await expect(page.getByTestId('help-log')).toHaveText('Ajudas usadas: nenhuma.');

	// The costs are in the saved exercise.
	await page.reload();
	await page.getByRole('button', { name: 'Retomar' }).click();
	await expect(summary(page)).toHaveText('FA: área 1, atraso 0,4 · HA: área 0,5, atraso 0,5');
	await expect(metrics(page)).toContainText('Atraso 1,5 TFA');

	// A new matrix keeps the costs in force.
	await build(page, 'A+B', { A: '2', B: '2' });
	await page.getByRole('dialog').getByRole('button', { name: 'Descartar' }).click();
	await expect(summary(page)).toHaveText('FA: área 1, atraso 0,4 · HA: área 0,5, atraso 0,5');

	await summary(page).click();
	await page.getByRole('button', { name: 'Restaurar o padrão' }).click();
	await expect(summary(page)).toHaveText('FA: área 1, atraso 1 · HA: área 0,5, atraso 0,5');
	await expect(page.getByRole('button', { name: 'Restaurar o padrão' })).toBeDisabled();
});

test('the optimum of a profile is searched again under new costs (GER-12, AJU-12)', async ({
	page
}) => {
	await page.goto('./');
	// e) {2,2; 1,1,1}: 1 FA + 1 HA by default; 4 HA once an HA costs 0,3 (1,2 < 1,3).
	await build(page, '{2,2; 1,1,1}');
	await page.getByRole('button', { name: 'Solução ótima', exact: true }).click();
	await show(page).click();
	await expect(page.getByTestId('optimal-summary')).toHaveText(
		'Solução ótima: 2 níveis; 1 FA, 1 HA; área 1,5 A_FA; atraso 1,5 T_FA.'
	);

	await summary(page).click();
	await setCost(page, 'Área do HA', '0,3');
	await setCost(page, 'Atraso do HA', '0,3');
	await page.getByRole('button', { name: 'Aplicar custos' }).click();
	await expect(page.getByTestId('optimal-summary')).toHaveText(
		'Solução ótima: 3 níveis; 0 FA, 4 HA; área 1,2 A_FA; atraso 0,9 T_FA.'
	);
	// Seen again, not counted again.
	const log = await page.evaluate(() => localStorage.getItem('dotree.exercise'));
	expect(JSON.parse(log!).help.optimal).toBe(1);
});
