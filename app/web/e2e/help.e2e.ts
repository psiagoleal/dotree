// Caminho relativo: app/web/e2e/help.e2e.ts
// MT-13: help on demand (spec 03). Nothing is revealed without a request and a confirmation
// (GER-01, AJU-02); results come before paths (GER-03); the solution leaves the student's
// circuit alone (AJU-03).
import { expect, test, type Page } from '@playwright/test';
import { r2ByMouse } from './r2';

test.use({ locale: 'pt-BR' });

const current = (page: Page) => page.getByTestId('level-current');
const point = (page: Page, column: number, signal: string) =>
	current(page)
		.getByRole('group', { name: new RegExp(`2\\^${column},`) })
		.getByRole('button', { name: signal, exact: true });
const keepTrying = (page: Page) => page.getByRole('button', { name: 'Vou tentar mais um pouco' });
const show = (page: Page) =>
	page.getByRole('dialog').getByRole('button', { name: 'Mostrar', exact: true });

/** Every answer the help can give, by where it shows or how it reads. */
async function expectNoAnswer(page: Page) {
	for (const id of [
		'hint',
		'next-move',
		'lower-bound',
		'lower-bound-path',
		'solution-summary',
		'solution',
		'wallace-summary',
		'wallace',
		'width-result',
		'width-path'
	]) {
		await expect(page.getByTestId(id)).toHaveCount(0);
	}
	await expect(page.locator('.dot.suggested')).toHaveCount(0);
	const text = (await page.locator('main').innerText()).replace(/\s+/g, ' ');
	expect(text).not.toMatch(/W = \d/);
	// The buttons name the methods; only an answer has the colon.
	expect(text).not.toMatch(
		/v_?max|Limite inferior:|Solução de (Dadda|Wallace):|Próxima jogada do método/
	);
	expect(text).not.toMatch(/faltam no mínimo|é a mais alta|pontos prontos|já pode ser fechado/);
}

test('ajuda-nao-aparece-sem-pedido (GER-01, AJU-02)', async ({ page }) => {
	await page.goto('./');
	await page.getByRole('button', { name: 'Soma de produtos' }).click();

	// Use the interface without asking for help.
	await page.keyboard.press('v');
	await page.keyboard.press('v');
	await page.getByText('Regras').click();
	await point(page, 2, 'a0_b2').click();
	await point(page, 2, 'a1_b1').click();
	await page.getByRole('button', { name: 'Formar célula' }).click();
	await page.getByRole('button', { name: 'Desfazer' }).click();
	await expectNoAnswer(page);

	// Each step of help: ask, the focus is on "keep trying", cancel, still nothing; confirm.
	const steps: [string, string][] = [
		['Dica', 'hint'],
		['Próxima jogada', 'next-move'],
		['Limite inferior', 'lower-bound'],
		['Solução de Dadda', 'solution-summary'],
		['Solução de Wallace', 'wallace-summary']
	];
	for (const [button] of steps) {
		await page.getByRole('button', { name: button, exact: true }).click();
		await expect(keepTrying(page)).toBeFocused();
		await page.keyboard.press('Escape');
		await expectNoAnswer(page);
	}
	for (const [button, answer] of steps) {
		await page.getByRole('button', { name: button, exact: true }).click();
		await show(page).click();
		await expect(page.getByTestId(answer)).toBeVisible();
	}
});

test('lower bound in two steps: the number, then the reasoning (GER-03, AJU-06)', async ({
	page
}) => {
	await page.goto('./');
	await page.getByRole('button', { name: 'Soma de produtos' }).click();
	await page.getByRole('button', { name: 'Limite inferior' }).click();
	await show(page).click();
	await expect(page.getByTestId('lower-bound')).toContainText('Limite inferior: 13.');
	await expect(page.getByTestId('lower-bound-path')).toHaveCount(0);

	await page.getByRole('button', { name: 'Ver a conta do limite' }).click();
	await expect(keepTrying(page)).toBeFocused();
	await show(page).click();
	await expect(page.getByTestId('lower-bound-path')).toContainText('20 − 7 = 13');
});

test('solution in two steps, without touching the circuit (GER-03, AJU-03, AJU-05)', async ({
	page
}) => {
	await page.goto('./');
	await page.getByRole('button', { name: 'Soma de produtos' }).click();
	await point(page, 2, 'a0_b2').click();
	await point(page, 2, 'a1_b1').click();
	await page.getByRole('button', { name: 'Formar célula' }).click();
	const metrics = await page.getByTestId('metrics').innerText();

	await page.getByRole('button', { name: 'Solução de Dadda' }).click();
	await show(page).click();
	await expect(page.getByTestId('solution-summary')).toContainText(
		'4 níveis e o CPA; 13 FA, 8 HA; área 17 A_FA'
	);
	await expect(page.getByTestId('solution')).toHaveCount(0);

	await page.getByRole('button', { name: 'Ver a solução de Dadda nível a nível' }).click();
	await show(page).click();
	const solution = page.getByTestId('solution');
	await expect(solution.locator('section.panel')).toHaveCount(6);
	await expect(solution.locator('.cell')).toHaveCount(21);

	// AJU-03: the student's circuit is as it was.
	await expect(page.getByTestId('metrics')).toHaveText(metrics);
	await expect(current(page).getByRole('group', { name: 'HA pendente' })).toBeVisible();
	await expect(page.getByRole('heading', { name: 'Nível 0 (atual)' })).toBeVisible();
});

test('using the solution replaces the circuit in one undo step and is logged (AJU-09)', async ({
	page
}) => {
	await page.goto('./');
	await page.getByRole('button', { name: 'Soma de produtos' }).click();
	await point(page, 2, 'a0_b2').click();
	await point(page, 2, 'a1_b1').click();
	await page.getByRole('button', { name: 'Formar célula' }).click();
	await expect(current(page).locator('.cell')).toHaveCount(1);

	const useIt = page.getByRole('button', { name: 'Usar a solução de Dadda' });
	await page.getByRole('button', { name: 'Solução de Dadda', exact: true }).click();
	await show(page).click();
	await expect(useIt).toHaveCount(0);
	await page.getByRole('button', { name: 'Ver a solução de Dadda nível a nível' }).click();
	await show(page).click();

	await useIt.click();
	await expect(keepTrying(page)).toBeFocused();
	await page.keyboard.press('Escape');
	await expect(page.getByTestId('complete')).toHaveCount(0);

	await useIt.click();
	await page.getByRole('dialog').getByRole('button', { name: 'Substituir' }).click();
	await expect(page.getByTestId('complete')).toBeVisible();
	await expect(page.getByTestId('metrics')).toContainText(/FA\s*13/);
	await expect(page.getByTestId('help-log')).toContainText('solução de Dadda aplicada: 1 vez');
	await page.getByRole('button', { name: 'Verificar circuito' }).click();
	await expect(page.getByTestId('verify-result')).toHaveText('16.384 casos, exaustivo, 0 erros.');
	await expect(page.getByRole('button', { name: 'Baixar todos (.zip)' })).toBeEnabled();

	// One undo gives the student's circuit back: level 0 with the one cell formed.
	await page.getByRole('button', { name: 'Desfazer' }).click();
	await expect(page.getByTestId('complete')).toHaveCount(0);
	await expect(current(page).locator('.cell')).toHaveCount(1);
	await expect(page.getByRole('heading', { name: /^Nível 0/ })).toBeVisible();
	await page.getByRole('button', { name: 'Refazer' }).click();
	await expect(page.getByTestId('complete')).toBeVisible();
});

test("Wallace's solution, applied, verifies and is logged apart (AJU-10)", async ({ page }) => {
	await page.goto('./');
	await page.getByRole('button', { name: 'Soma de produtos' }).click();

	await page.getByRole('button', { name: 'Solução de Wallace', exact: true }).click();
	await show(page).click();
	await expect(page.getByTestId('wallace-summary')).toContainText(
		'4 níveis e o CPA; 13 FA, 3 HA; área 14,5 A_FA'
	);
	await page.getByRole('button', { name: 'Ver a solução de Wallace nível a nível' }).click();
	await show(page).click();
	// R1 by hand (spec 03): 4 FA, 4 FA, 2 FA, 2 FA + 1 HA, and the CPA with 1 FA + 2 HA.
	const wallace = page.getByTestId('wallace');
	for (const [level, cells] of [4, 4, 2, 3, 3].entries()) {
		await expect(page.getByTestId(`wallace-level-${level}`).locator('.cell')).toHaveCount(cells);
	}
	await expect(wallace.locator('.cell')).toHaveCount(16);

	await page.getByRole('button', { name: 'Usar a solução de Wallace' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Substituir' }).click();
	await expect(page.getByTestId('complete')).toBeVisible();
	await expect(page.getByTestId('metrics')).toContainText(/HA\s*3/);
	await page.getByRole('button', { name: 'Verificar circuito' }).click();
	await expect(page.getByTestId('verify-result')).toHaveText('16.384 casos, exaustivo, 0 erros.');
	const log = page.getByTestId('help-log');
	await expect(log).toContainText('solução de Wallace: 1 (nível a nível: 1)');
	await expect(log).toContainText('solução de Wallace aplicada: 1 vez');
	await expect(log).not.toContainText('solução de Dadda');
});

test('comparing R2 with Dadda and Wallace: only what was revealed, nothing counted (MT-38)', async ({
	page
}) => {
	await page.goto('./');
	await r2ByMouse(page);
	await page.getByRole('button', { name: 'Fechar com CPA' }).click();
	const comparison = page.getByTestId('comparison');
	await expect(comparison.getByTestId('comparison-empty')).toBeVisible();

	for (const method of ['Dadda', 'Wallace']) {
		await page.getByRole('button', { name: `Solução de ${method}`, exact: true }).click();
		await show(page).click();
	}
	const log = await page.getByTestId('help-log').innerText();
	const rows = comparison.getByTestId('comparison-table').locator('tr');
	await expect(rows.nth(0)).toHaveText(/Seu circuito\s*Dadda \(clássico\)\s*Wallace/);
	await expect(rows.nth(1)).toHaveText(/Área \(A_FA\)\s*15\s*17\s*14,5/);
	await expect(rows.nth(2)).toHaveText(/Atraso \(T_FA\)\s*6\s*7\s*6/);
	await expect(rows.nth(4)).toHaveText(/HA\s*4\s*8\s*3/);
	await expect(rows.nth(9)).toHaveText(/Área × atraso\s*90\s*119\s*87/);
	await expect(rows.last()).toHaveText(/Não dominada\s*não\s*não\s*sim/);
	await expect(comparison.locator('[data-testid^="height-table-"]')).toHaveCount(0);

	// Wallace's levels bring its height table into the comparison.
	await page.getByRole('button', { name: 'Ver a solução de Wallace nível a nível' }).click();
	await show(page).click();
	await expect(comparison.getByTestId('height-table-wallace').locator('tbody tr')).toHaveCount(6);
	await expect(comparison.getByTestId('height-table-dadda')).toHaveCount(0);
	// The comparison itself counts nothing: only the Wallace path was added to the log.
	await expect(page.getByTestId('help-log')).not.toHaveText(log);
	await expect(page.getByTestId('help-log')).toContainText(
		'solução de Wallace: 1 (nível a nível: 1)'
	);
	await expect(page.getByTestId('help-log')).toContainText(
		'solução de Dadda: 1 (nível a nível: 0)'
	);
});
