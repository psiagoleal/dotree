// Caminho relativo: app/web/e2e/counters.e2e.ts
// MT-32b (CMP-20, ADR 0005): a 7:3 counter through the interface, by keyboard, up to the
// verification and the VHDL; and the accessibility audit with a counter on the screen.
import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';
import { point } from './r2';

test.use({ locale: 'pt-BR' });

const SEVEN = ['A', 'B', 'C', 'D', 'E', 'F', 'G'];

/** A + B + … + G, one bit each: seven points in 2^0, W = 3. */
async function seven(page: Page) {
	await page.goto('./');
	await page.getByLabel('Expressão').fill(SEVEN.join(' + '));
	for (const v of SEVEN) await page.locator(`input[name="width-${v}"]`).fill('1');
	await page.getByRole('button', { name: 'Montar a matriz' }).click();
}

/** Space selects the focused point and F forms the cell (spec 02, keyboard). */
async function sevenByKeyboard(page: Page) {
	for (const v of SEVEN) {
		await point(page, 0, `${v.toLowerCase()}0`).focus();
		await page.keyboard.press('Space');
	}
	await expect(page.getByText('Seleção: 7')).toBeVisible();
	await page.keyboard.press('f');
}

test('a 7:3 counter by keyboard: one cell adds seven bits, verified and in the VHDL (CMP-20, VHD-11)', async ({
	page
}) => {
	await seven(page);
	// Six points are refused with the rule, which names the counters.
	for (const v of SEVEN.slice(0, 6)) await point(page, 0, `${v.toLowerCase()}0`).click();
	await page.getByRole('button', { name: 'Formar célula' }).click();
	await expect(page.getByRole('alert')).toHaveText(
		'Um FA usa 3 pontos; um HA, 2; um contador 7:3, 7; e um contador 15:4, 15. A seleção tem 6.'
	);
	await page.getByRole('button', { name: 'Limpar seleção' }).click();

	await sevenByKeyboard(page);
	await expect(page.getByTestId('metrics')).toContainText(/7:3\s*1/);
	await page.getByRole('button', { name: 'Concluir nível' }).click();
	await expect(page.getByTestId('complete')).toBeVisible();
	await expect(page.getByTestId('metrics')).toContainText(/Área\s*4\s/);
	await expect(page.getByTestId('metrics')).toContainText(/Atraso\s*3\s/);

	await expect(page.locator('svg [data-cell="1"] title')).toHaveText('C73_1');
	await expect(page.getByTestId('wiring-table').getByRole('row', { name: /C73_1/ })).toContainText(
		'a0, b0, c0, d0, e0, f0, g0'
	);

	// The comparison gets the counter rows: the student used one, Dadda none.
	await page.getByRole('button', { name: 'Solução de Dadda', exact: true }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Mostrar', exact: true }).click();
	const row = page.getByTestId('comparison-table').getByRole('row', { name: /^7:3/ });
	await expect(row.getByRole('cell')).toHaveText(['1', '0']);
	await expect(
		page.getByTestId('comparison-table').getByRole('row', { name: /^15:4/ })
	).toBeVisible();

	await page.getByRole('button', { name: 'Verificar circuito' }).click();
	await expect(page.getByTestId('verify-result')).toHaveText('128 casos, exaustivo, 0 erros.');
	await expect(page.locator('pre[data-file="counter_7_3.vhd"]')).toContainText(
		'entity counter_7_3 is'
	);
	await expect(page.locator('pre[data-file="dotree_circuito.vhd"]')).toContainText(
		'C73_1 : counter_7_3 port map'
	);
});

for (const scheme of ['light', 'dark'] as const) {
	test(`no serious or critical accessibility violation with a counter, ${scheme} theme (GER-07)`, async ({
		page
	}) => {
		await page.emulateMedia({ colorScheme: scheme });
		await seven(page);
		await sevenByKeyboard(page);
		await page.getByRole('button', { name: 'Concluir nível' }).click();
		await page.getByTestId('costs-summary').click();
		const { violations } = await new AxeBuilder({ page }).analyze();
		const serious = violations
			.filter((v) => v.impact === 'serious' || v.impact === 'critical')
			.map((v) => `${v.id} (${v.impact}) ${v.nodes.map((n) => n.target.join(' ')).join(' | ')}`);
		expect(serious).toEqual([]);
	});
}
