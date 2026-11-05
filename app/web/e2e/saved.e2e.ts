// Caminho relativo: app/web/e2e/saved.e2e.ts
// MT-15b: the exercise is saved in local storage and offered back on the next visit
// (SAL-01); blocked or full storage is reported once (SAL-04); a corrupted saved state is
// offered as a download before it is discarded (SAL-05).
import { readFileSync } from 'node:fs';
import { expect, test, type Page } from '@playwright/test';
import { formByMouse } from './r2';

test.use({ locale: 'pt-BR' });

const KEY = 'dotree.exercise';
const offer = (page: Page) => page.getByTestId('saved-offer');
const corrupt = (page: Page) => page.getByTestId('saved-corrupt');
const warning = (page: Page) => page.getByTestId('storage-warning');
/** Everything the exercise shows, minus the announcement of the last action. */
const exerciseText = async (page: Page) =>
	(await page.locator('section.result').innerText()).replace(/\n*^[FH]A formad[oa]\.$\n*/m, '\n\n');

async function someWork(page: Page) {
	await page.goto('./');
	await page.getByRole('button', { name: 'Soma de produtos' }).click();
	await page.getByLabel('Bits da saída').fill('7');
	await page.getByRole('button', { name: 'Responder' }).click();
	await formByMouse(page, 1, ['a0_b1', 'a1_b0', 'c0_d1']);
	await formByMouse(page, 1, ['c1_d0', 'e1', 'e0']);
	await formByMouse(page, 2, ['a0_b2', 'a1_b1']);
	await page.getByRole('button', { name: 'Concluir nível' }).click();
	await formByMouse(page, 1, ['f0', 's1', 's2']);
}

/** Puts `text` in the saved slot before the page's scripts run, on the first load only. */
async function plant(page: Page, text: string) {
	await page.addInitScript(
		([key, value]) => {
			if (sessionStorage.getItem('planted')) return;
			sessionStorage.setItem('planted', '1');
			localStorage.setItem(key, value);
		},
		[KEY, text]
	);
}

test('closing the tab loses nothing: the next visit offers to resume (SAL-01)', async ({
	page
}) => {
	await someWork(page);
	const before = await exerciseText(page);
	await page.reload();
	await expect(offer(page)).toContainText(
		'Há um exercício salvo neste navegador: A*B + C*D + 3(E+F) + G.'
	);
	await expect(page.locator('section.result')).toHaveCount(0);
	await page.getByRole('button', { name: 'Retomar' }).click();
	await expect(offer(page)).toHaveCount(0);
	await expect(page.getByLabel('Expressão')).toHaveValue('A*B + C*D + 3(E+F) + G');
	expect(await exerciseText(page)).toBe(before);

	// The resumed exercise keeps saving: one more cell survives another reload.
	await formByMouse(page, 2, ['a2_b0', 'c1_d1', 'e1']);
	const after = await exerciseText(page);
	await page.reload();
	await page.getByRole('button', { name: 'Retomar' }).click();
	expect(await exerciseText(page)).toBe(after);
	await expect(warning(page)).toHaveCount(0);
});

test('the saved exercise is discarded only after confirmation', async ({ page }) => {
	await someWork(page);
	await page.reload();
	// Building another matrix while the offer stands asks first.
	await page.getByRole('button', { name: 'Uma variável' }).click();
	await page.getByRole('button', { name: 'Manter o exercício salvo' }).click();
	await expect(offer(page)).toBeVisible();
	await expect(page.locator('section.result')).toHaveCount(0);

	await offer(page).getByRole('button', { name: 'Descartar' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Descartar' }).click();
	await expect(offer(page)).toHaveCount(0);
	expect(await page.evaluate((k) => localStorage.getItem(k), KEY)).toBeNull();
	await page.reload();
	await expect(page.getByRole('button', { name: 'Soma de produtos' })).toBeVisible();
	await expect(offer(page)).toHaveCount(0);
});

test('a blocked storage is reported once and the app works normally (SAL-04)', async ({ page }) => {
	await page.addInitScript(() => {
		Object.defineProperty(window, 'localStorage', {
			get() {
				throw new DOMException('blocked', 'SecurityError');
			}
		});
	});
	await someWork(page);
	await expect(warning(page)).toHaveCount(1);
	await expect(warning(page)).toContainText('o progresso não será lembrado');
	await expect(page.getByTestId('level-0')).toBeVisible();
	await page.reload();
	await expect(offer(page)).toHaveCount(0);
});

test('a full storage is reported once, at the first save that fails (SAL-04)', async ({ page }) => {
	await page.addInitScript(() => {
		Storage.prototype.setItem = () => {
			throw new DOMException('full', 'QuotaExceededError');
		};
	});
	await page.goto('./');
	await expect(page.getByRole('button', { name: 'Soma de produtos' })).toBeVisible();
	await expect(warning(page)).toHaveCount(0);
	await page.getByRole('button', { name: 'Soma de produtos' }).click();
	await expect(warning(page)).toHaveCount(1);
	await formByMouse(page, 1, ['a0_b1', 'a1_b0', 'c0_d1']);
	await expect(warning(page)).toHaveCount(1);
});

for (const [name, text, reason] of [
	['an old schema', '{"format": "dotree-exercise", "schema": 0}', /esquema 0, antigo demais/],
	['broken JSON', '{"format": "dotree-exer', /não é um JSON de exercício válido/]
] as const) {
	test(`a corrupted saved state (${name}) is offered as a download before it goes (SAL-05)`, async ({
		page
	}, info) => {
		await plant(page, text);
		await page.goto('./');
		await expect(corrupt(page)).toContainText(
			'O exercício salvo neste navegador não pôde ser aberto:'
		);
		await expect(corrupt(page)).toContainText(reason);
		await expect(offer(page)).toHaveCount(0);
		// The app works without it, and nothing overwrites it while the choice is open.
		await expect(page.getByRole('button', { name: 'Montar a matriz' })).toBeEnabled();

		const downloading = page.waitForEvent('download');
		await page.getByRole('button', { name: 'Baixar o conteúdo salvo' }).click();
		const file = await downloading;
		expect(file.suggestedFilename()).toBe('dotree-salvo.json');
		const path = info.outputPath('saved.json');
		await file.saveAs(path);
		expect(readFileSync(path, 'utf8')).toBe(text);
		expect(await page.evaluate((k) => localStorage.getItem(k), KEY)).toBe(text);

		await corrupt(page).getByRole('button', { name: 'Descartar' }).click();
		await page.getByRole('dialog').getByRole('button', { name: 'Descartar' }).click();
		await expect(corrupt(page)).toHaveCount(0);
		expect(await page.evaluate((k) => localStorage.getItem(k), KEY)).toBeNull();
		await page.getByRole('button', { name: 'Soma de produtos' }).click();
		await expect(page.getByTestId('level-current')).toBeVisible();
	});
}
