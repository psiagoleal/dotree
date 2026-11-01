// Caminho relativo: app/web/e2e/compression.e2e.ts
// MT-12a: the tree of R2 (spec 00) built through the interface, up to the CPA.
import { expect, test, type Page } from '@playwright/test';

test.use({ locale: 'pt-BR' });

const current = (page: Page) => page.getByTestId('level-current');

const point = (page: Page, column: number, signal: string) =>
	current(page)
		.getByRole('group', { name: new RegExp(`2\\^${column},`) })
		.getByRole('button', { name: signal, exact: true });

async function formByMouse(page: Page, column: number, signals: string[]) {
	for (const s of signals) await point(page, column, s).click();
	await page.getByRole('button', { name: 'Formar célula' }).click();
}

/** Keyboard only: focus each point, Space selects it, F forms the cell (GER-07). */
async function formByKeyboard(page: Page, column: number, signals: string[]) {
	for (const s of signals) {
		await point(page, column, s).focus();
		await page.keyboard.press('Space');
	}
	await page.keyboard.press('f');
}

test('R2 tree through the interface: panels, numbered cells and metrics (CMP-05, CMP-13 to CMP-15)', async ({
	page
}) => {
	await page.goto('./');
	await page.getByRole('button', { name: 'Soma de produtos' }).click();
	const conclude = page.getByRole('button', { name: 'Concluir nível' });

	await formByMouse(page, 1, ['a0_b1', 'a1_b0', 'c0_d1']);
	await formByMouse(page, 1, ['c1_d0', 'e1', 'e0']);
	await formByMouse(page, 2, ['a0_b2', 'a1_b1']);
	await conclude.click();

	await formByMouse(page, 1, ['f0', 's1', 's2']);
	await formByMouse(page, 2, ['a2_b0', 'c1_d1', 'e1']);
	await formByMouse(page, 2, ['s3', 'k1', 'k2']);
	await formByMouse(page, 3, ['a1_b2', 'a2_b1', 'k3']);
	await conclude.click();

	await formByKeyboard(page, 0, ['a0_b0', 'c0_d0', 'e0']);
	await formByKeyboard(page, 2, ['s5', 'k4', 's6']);
	await formByKeyboard(page, 3, ['k5', 's7', 'k6']);
	await formByKeyboard(page, 4, ['a2_b2', 'k7']);
	await conclude.click();

	// One panel per concluded level, cells numbered as in spec 00.
	const cells: [number, string[]][] = [
		[0, ['FA 1', 'FA 2', 'HA 3']],
		[1, ['FA 4', 'FA 5', 'FA 6', 'FA 7']],
		[2, ['FA 8', 'FA 9', 'FA 10', 'HA 11']]
	];
	for (const [level, names] of cells) {
		const panel = page.getByTestId(`level-${level}`);
		for (const name of names) {
			await expect(panel.getByRole('group', { name, exact: true })).toBeVisible();
		}
		await expect(panel.locator('.cell')).toHaveCount(names.length);
	}
	await expect(page.getByRole('heading', { name: 'Nível 3 (atual)' })).toBeVisible();

	// Heights of the level that the CPA will close (CMP-13); W is still hidden, so 2^6 is not drawn.
	for (const [column, height] of [
		[5, 1],
		[4, 2],
		[3, 2],
		[2, 1],
		[1, 2],
		[0, 3]
	] as const) {
		await expect(
			current(page).getByRole('group', { name: new RegExp(`2\\^${column},`) })
		).toHaveAccessibleName(`Coluna de peso 2^${column}, ${height} pontos`);
	}
	await expect(current(page).getByRole('group', { name: /2\^6,/ })).toHaveCount(0);

	// 9 FA, 2 HA, area 10 A_FA, delay 3 T_FA (FA9 and FA10), counted by hand.
	const metrics = page.getByTestId('metrics');
	await expect(metrics).toContainText(/FA\s*9/);
	await expect(metrics).toContainText(/HA\s*2/);
	await expect(metrics).toContainText(/AND\s*13/);
	await expect(metrics).toContainText(/Área\s*10\s*AFA/);
	await expect(metrics).toContainText(/Atraso\s*3\s*TFA/);

	// Undo reaches back into a concluded level.
	await page.getByRole('button', { name: 'Desfazer' }).click();
	await expect(page.getByRole('heading', { name: 'Nível 2 (atual)' })).toBeVisible();
	await expect(current(page).getByRole('group', { name: 'HA pendente' })).toBeVisible();
});

test('a refusal names the rule, not the fix (GER-02, CMP-06)', async ({ page }) => {
	await page.goto('./');
	await page.getByRole('button', { name: 'Soma de produtos' }).click();
	await point(page, 0, 'a0_b0').click();
	await point(page, 1, 'a0_b1').click();
	await page.getByRole('button', { name: 'Formar célula' }).click();
	await expect(page.getByRole('alert')).toHaveText('Uma célula soma bits do mesmo peso.');
});
