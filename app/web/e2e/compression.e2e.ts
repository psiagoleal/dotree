// Caminho relativo: app/web/e2e/compression.e2e.ts
// MT-12a and MT-12b: the tree of R2 (spec 00) built through the interface, the CPA, the
// result and the questions asked before discarding work (CMP-04, CMP-09, EXP-04).
import { expect, test, type Locator, type Page } from '@playwright/test';

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

/**
 * Moves the focus to `target` with Tab or Shift+Tab only, as a keyboard user would: no
 * programmatic focus (GER-07).
 */
async function tabTo(page: Page, target: Locator) {
	for (let i = 0; i < 300; i++) {
		const where = await target.evaluate((el) => {
			const a = document.activeElement;
			if (a === el) return 'here';
			if (!a || a === document.body) return 'after';
			return a.compareDocumentPosition(el) & Node.DOCUMENT_POSITION_FOLLOWING ? 'after' : 'before';
		});
		if (where === 'here') return;
		await page.keyboard.press(where === 'after' ? 'Tab' : 'Shift+Tab');
	}
	throw new Error('the target cannot be reached with Tab');
}

/** Tab to each point and select it with Space; F forms the cell. */
async function formByTab(page: Page, column: number, signals: string[]) {
	for (const s of signals) {
		await tabTo(page, point(page, column, s));
		await page.keyboard.press('Space');
	}
	await page.keyboard.press('f');
}

async function press(page: Page, name: string) {
	await tabTo(page, page.getByRole('button', { name, exact: true }));
	await page.keyboard.press('Enter');
}

/** Levels 0 to 2 of the R2 tree by mouse, leaving level 3 ready for the CPA. */
async function r2ByMouse(page: Page) {
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
	await formByMouse(page, 0, ['a0_b0', 'c0_d0', 'e0']);
	await formByMouse(page, 2, ['s5', 'k4', 's6']);
	await formByMouse(page, 3, ['k5', 's7', 'k6']);
	await formByMouse(page, 4, ['a2_b2', 'k7']);
	await conclude.click();
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

test('R2 completed by keyboard alone: CPA, result and critical path (MT-12b, CMP-10 to CMP-15, GER-07)', async ({
	page
}) => {
	test.setTimeout(120_000);
	await page.goto('./');
	await press(page, 'Soma de produtos');

	const cpa = page.getByRole('button', { name: 'Fechar com CPA' });
	await expect(cpa).toBeDisabled();
	await expect(
		page.getByText('A coluna 2^0 tem 5 pontos; o CPA aceita no máximo 3.')
	).toBeVisible();

	const levels: [number, string[]][][] = [
		[
			[1, ['a0_b1', 'a1_b0', 'c0_d1']],
			[1, ['c1_d0', 'e1', 'e0']],
			[2, ['a0_b2', 'a1_b1']]
		],
		[
			[1, ['f0', 's1', 's2']],
			[2, ['a2_b0', 'c1_d1', 'e1']],
			[2, ['s3', 'k1', 'k2']],
			[3, ['a1_b2', 'a2_b1', 'k3']]
		],
		[
			[0, ['a0_b0', 'c0_d0', 'e0']],
			[2, ['s5', 'k4', 's6']],
			[3, ['k5', 's7', 'k6']],
			[4, ['a2_b2', 'k7']]
		]
	];
	for (const cells of levels) {
		for (const [column, signals] of cells) await formByTab(page, column, signals);
		await press(page, 'Concluir nível');
	}
	await expect(cpa).toBeEnabled();
	await press(page, 'Fechar com CPA');

	// CMP-11: FA, FA, HA, FA, FA, HA from 2^0 to 2^5; the carry of HA17 is R6.
	await expect(page.getByRole('heading', { name: 'Nível 3: CPA' })).toBeVisible();
	const cpaPanel = page.getByTestId('level-3');
	for (const name of ['FA 12', 'FA 13', 'HA 14', 'FA 15', 'FA 16', 'HA 17']) {
		await expect(cpaPanel.getByRole('group', { name, exact: true })).toBeVisible();
	}
	await expect(page.locator('section.panel')).toHaveCount(5);
	await expect(page.locator('.cell')).toHaveCount(17);

	// CMP-12: complete, with R0 to R6.
	await expect(page.getByTestId('complete')).toHaveText(
		'Circuito completo: o resultado tem 7 bits, de R0 a R6.'
	);
	for (let j = 0; j <= 6; j++) {
		await expect(
			page.getByTestId('result').getByRole('group', { name: `Coluna de peso 2^${j}, bit R${j}` })
		).toBeVisible();
	}

	// GER-04, CMP-14, CMP-15: 13 FA, 4 HA, 13 AND, 15 A_FA, 6 T_FA.
	const metrics = page.getByTestId('metrics');
	await expect(metrics).toContainText(/FA\s*13/);
	await expect(metrics).toContainText(/HA\s*4/);
	await expect(metrics).toContainText(/AND\s*13/);
	await expect(metrics).toContainText(/Área\s*15\s*AFA/);
	await expect(metrics).toContainText(/Atraso\s*6\s*TFA/);

	// CMP-15: the critical path only on request.
	await expect(page.locator('.cell.critical')).toHaveCount(0);
	await press(page, 'Destacar caminho crítico');
	for (const n of [1, 2, 4, 6, 8, 9, 12, 13, 14, 15, 16, 17]) {
		await expect(
			page.getByRole('group', { name: new RegExp(`^[FH]A ${n}, no caminho crítico$`) })
		).toBeVisible();
	}
	await expect(page.locator('.cell.critical')).toHaveCount(12);
});

test('going back to a level asks before discarding; undo brings it back (CMP-04)', async ({
	page
}) => {
	await page.goto('./');
	await r2ByMouse(page);
	const back = page.getByTestId('level-0').getByRole('button', { name: 'Voltar a este nível' });

	await back.click();
	await expect(page.getByRole('dialog', { name: 'Voltar ao nível 0?' })).toBeVisible();
	await expect(page.getByRole('button', { name: 'Ficar onde estou' })).toBeFocused();
	await page.keyboard.press('Escape');
	await expect(page.getByRole('heading', { name: 'Nível 3 (atual)' })).toBeVisible();

	await back.click();
	await page.getByRole('button', { name: 'Voltar', exact: true }).click();
	await expect(page.getByRole('heading', { name: 'Nível 0 (atual)' })).toBeVisible();
	await expect(current(page).getByRole('group', { name: /pendente/ })).toHaveCount(3);

	await page.keyboard.press('Control+z');
	await expect(page.getByRole('heading', { name: 'Nível 3 (atual)' })).toBeVisible();
});

test('building another matrix asks before discarding the compression (EXP-04)', async ({
	page
}) => {
	await page.goto('./');
	await page.getByRole('button', { name: 'Soma de produtos' }).click();
	await formByMouse(page, 2, ['a0_b2', 'a1_b1']);
	const pending = current(page).getByRole('group', { name: 'HA pendente' });
	await expect(pending).toBeVisible();

	const buildButton = page.getByRole('button', { name: 'Montar a matriz' });
	await buildButton.click();
	const keep = page.getByRole('button', { name: 'Continuar a compressão' });
	await expect(keep).toBeFocused();
	await keep.click();
	await expect(pending).toBeVisible();

	await buildButton.click();
	await page.getByRole('button', { name: 'Descartar e montar' }).click();
	await expect(pending).toHaveCount(0);
	await expect(page.getByRole('heading', { name: 'Nível 0 (atual)' })).toBeVisible();

	// With nothing done yet, there is nothing to lose: no question.
	await page.getByRole('button', { name: 'Multiplicador 4×4' }).click();
	await expect(page.getByRole('dialog')).toHaveCount(0);
	await expect(page.getByRole('heading', { name: 'Nível 0 (atual)' })).toBeVisible();
});

test('the dropped carry is explained only on request and after confirmation (CMP-09)', async ({
	page
}) => {
	await page.goto('./');
	await page.getByLabel('Expressão').fill('A + B + C');
	for (const v of ['A', 'B', 'C']) await page.locator(`input[name="width-${v}"]`).fill('1');
	await page.getByRole('button', { name: 'Montar a matriz' }).click();

	const conclude = page.getByRole('button', { name: 'Concluir nível' });
	await formByMouse(page, 0, ['a0', 'b0']);
	await conclude.click();
	await formByMouse(page, 0, ['c0', 's1']);
	await conclude.click();
	await formByMouse(page, 1, ['k1', 'k2']);
	await conclude.click();

	await expect(page.getByRole('group', { name: 'HA 3, vai-um sempre 0' })).toBeVisible();
	await expect(page.getByTestId('dropped-explanation')).toHaveCount(0);
	await expect(page.getByText(/W = 2/)).toHaveCount(0);

	await page.getByRole('button', { name: 'Por quê?' }).click();
	await expect(page.getByRole('button', { name: 'Vou tentar mais um pouco' })).toBeFocused();
	await page.keyboard.press('Escape');
	await expect(page.getByText(/W = 2/)).toHaveCount(0);

	await page.getByRole('button', { name: 'Por quê?' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Mostrar', exact: true }).click();
	await expect(page.getByTestId('dropped-explanation')).toContainText('W = 2');
});
