// Caminho relativo: app/web/e2e/file.e2e.ts
// MT-15a: export an exercise as JSON and open it again (SAL-02); refused files leave the
// open exercise as it was (SAL-03).
import { readFileSync } from 'node:fs';
import { expect, test, type Page } from '@playwright/test';
import { formByMouse } from './r2';

test.use({ locale: 'pt-BR' });

const show = (page: Page) =>
	page.getByRole('dialog').getByRole('button', { name: 'Mostrar', exact: true });
/** Everything the exercise shows, minus the announcement of the last action. */
const exercise = (page: Page) => ({
	innerText: async () =>
		(await page.locator('section.result').innerText()).replace(
			/\n*^[FH]A formad[oa]\.$\n*/m,
			'\n\n'
		)
});
const status = (page: Page) => page.getByTestId('file-status');

/** R2 levels 0 and 1, two cells pending on level 2, W answered with its working, and the
 *  lower bound revealed. */
async function midExercise(page: Page) {
	await page.goto('./');
	await page.getByRole('button', { name: 'Soma de produtos' }).click();
	await page.getByLabel('Bits da saída').fill('6');
	await page.getByRole('button', { name: 'Responder' }).click();
	await page.getByRole('button', { name: 'Mostrar a conta' }).click();
	await show(page).click();
	await page.getByRole('button', { name: 'Limite inferior' }).click();
	await show(page).click();
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
	await formByMouse(page, 4, ['a2_b2', 'k7']);
}

async function exportFile(page: Page, path: string): Promise<string> {
	const downloading = page.waitForEvent('download');
	await page.getByRole('button', { name: 'Exportar exercício (.json)' }).click();
	const file = await downloading;
	expect(file.suggestedFilename()).toBe('dotree-exercicio.json');
	await file.saveAs(path);
	return readFileSync(path, 'utf8');
}

async function openFile(page: Page, name: string, text: string) {
	await page.getByTestId('file-input').setInputFiles({
		name,
		mimeType: 'application/json',
		buffer: Buffer.from(text)
	});
}

test('export, reload and open give back the same exercise (SAL-02)', async ({ page }, info) => {
	await midExercise(page);
	const before = await exercise(page).innerText();
	const text = await exportFile(page, info.outputPath('exercise.json'));
	const file = JSON.parse(text);
	expect(file).toMatchObject({
		format: 'dotree-exercise',
		schema: 1,
		expression: 'A*B + C*D + 3(E+F) + G',
		widths: { A: 3, B: 3, C: 2, D: 2, E: 2, F: 1, G: 1 },
		reveals: { width: { answer: 6, path: true } },
		help: { lowerBound: 1, lowerBoundPath: 0 }
	});
	expect(file.levels).toHaveLength(2);
	expect(file.pending).toHaveLength(2);

	await page.reload();
	await expect(page.getByRole('button', { name: 'Exportar exercício (.json)' })).toBeDisabled();
	await openFile(page, 'meu.json', text);
	await expect(status(page)).toHaveText('meu.json aberto.');
	await expect(page.getByLabel('Expressão')).toHaveValue('A*B + C*D + 3(E+F) + G');
	await expect(page.locator('input[name="width-E"]')).toHaveValue('2');
	await expect(page.getByTestId('lower-bound')).toBeVisible();
	await expect(page.getByTestId('width-path')).toBeVisible();
	expect(await exercise(page).innerText()).toBe(before);
	// The reopened exercise goes on from where it was.
	await formByMouse(page, 2, ['s5', 'k4', 's6']);
	await formByMouse(page, 3, ['k5', 's7', 'k6']);
	await page.getByRole('button', { name: 'Concluir nível' }).click();
	await page.getByRole('button', { name: 'Fechar com CPA' }).click();
	await expect(page.getByTestId('metrics')).toContainText(/FA\s*13/);
	await expect(page.getByTestId('metrics')).toContainText(/HA\s*4/);
});

test('bad files are refused with the reason, leaving the exercise as it was (SAL-03)', async ({
	page
}, info) => {
	await midExercise(page);
	const text = await exportFile(page, info.outputPath('exercise.json'));
	const before = await exercise(page).innerText();
	const file = JSON.parse(text);
	const cases: [string, string, RegExp][] = [
		['quebrado.json', '{"format": "dotree-exer', /não é um JSON de exercício válido/],
		['outro.json', '{"name": "x"}', /não é um exercício do dotree/],
		['futuro.json', JSON.stringify({ ...file, schema: 2 }), /esquema 2, de uma versão mais nova/],
		[
			'celula.json',
			JSON.stringify({ ...file, pending: [{ kind: 'ha', column: 0, inputs: [0, 99] }] }),
			/a célula 1 do nível 2 não obedece às regras/
		]
	];
	for (const [name, contents, reason] of cases) {
		await openFile(page, name, contents);
		await expect(status(page)).toHaveText(new RegExp(`^Não foi possível abrir ${name}: `));
		await expect(status(page)).toHaveText(reason);
		await expect(status(page)).toContainText('O exercício aberto não mudou.');
		expect(await exercise(page).innerText()).toBe(before);
	}
});

test('opening a file over a compression in progress asks first (EXP-04)', async ({
	page
}, info) => {
	await midExercise(page);
	const text = await exportFile(page, info.outputPath('exercise.json'));
	await page.getByRole('button', { name: 'Desfazer' }).click();
	const before = await exercise(page).innerText();

	await openFile(page, 'meu.json', text);
	await page.getByRole('button', { name: 'Continuar a compressão' }).click();
	expect(await exercise(page).innerText()).toBe(before);
	await expect(status(page)).toHaveText('');

	await openFile(page, 'meu.json', text);
	await page.getByRole('button', { name: 'Descartar e montar' }).click();
	await expect(status(page)).toHaveText('meu.json aberto.');
	expect(await exercise(page).innerText()).not.toBe(before);
});
