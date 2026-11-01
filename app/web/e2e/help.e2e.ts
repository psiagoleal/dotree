// Caminho relativo: app/web/e2e/help.e2e.ts
// MT-13: help on demand (spec 03). Nothing is revealed without a request and a confirmation
// (GER-01, AJU-02); results come before paths (GER-03); the solution leaves the student's
// circuit alone (AJU-03).
import { expect, test, type Page } from '@playwright/test';

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
		'width-result',
		'width-path'
	]) {
		await expect(page.getByTestId(id)).toHaveCount(0);
	}
	await expect(page.locator('.dot.suggested')).toHaveCount(0);
	const text = (await page.locator('main').innerText()).replace(/\s+/g, ' ');
	expect(text).not.toMatch(/W = \d/);
	expect(text).not.toMatch(/v_?max|Limite inferior:|Solução de Dadda|Próxima jogada do método/);
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
		['Solução automática', 'solution-summary']
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

	await page.getByRole('button', { name: 'Solução automática' }).click();
	await show(page).click();
	await expect(page.getByTestId('solution-summary')).toContainText(
		'4 níveis e o CPA; 13 FA, 8 HA; área 17 A_FA'
	);
	await expect(page.getByTestId('solution')).toHaveCount(0);

	await page.getByRole('button', { name: 'Ver a solução nível a nível' }).click();
	await show(page).click();
	const solution = page.getByTestId('solution');
	await expect(solution.locator('section.panel')).toHaveCount(6);
	await expect(solution.locator('.cell')).toHaveCount(21);

	// AJU-03: the student's circuit is as it was.
	await expect(page.getByTestId('metrics')).toHaveText(metrics);
	await expect(current(page).getByRole('group', { name: 'HA pendente' })).toBeVisible();
	await expect(page.getByRole('heading', { name: 'Nível 0 (atual)' })).toBeVisible();
});
