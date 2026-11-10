// Caminho relativo: app/web/e2e/guide.e2e.ts
// GER-11: "How to use" opens from the header at any time and width, counts nothing, and
// the page shortcuts stay quiet while it is open.
import { expect, test, type Page } from '@playwright/test';

test.use({ locale: 'pt-BR' });

const opener = (page: Page) => page.getByRole('button', { name: 'Como usar' });
const guide = (page: Page) => page.getByRole('dialog', { name: 'Como usar o dotree' });
/** The help counts of the exercise saved in the browser after every action (SAL-01). */
const savedHelp = (page: Page) =>
	page.evaluate(() => JSON.parse(localStorage.getItem('dotree.exercise') ?? 'null')?.help);

test('in the middle of an exercise, the guide counts nothing and stops the shortcuts (GER-11)', async ({
	page
}) => {
	await page.goto('./');
	await page.getByRole('button', { name: 'Soma de produtos' }).click();
	await page.getByRole('button', { name: 'Dica', exact: true }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Mostrar', exact: true }).click();
	await expect(page.getByTestId('hint')).toBeVisible();
	const before = await savedHelp(page);
	expect(before.hints).toBe(1);

	await opener(page).click();
	await expect(guide(page)).toBeVisible();
	for (const title of ['Passos', 'Teclado', 'Ajudas', 'Limitações', 'Referências'])
		await expect(guide(page).getByRole('heading', { name: title })).toBeVisible();
	await page.keyboard.press('v');
	await page.keyboard.press('Escape');
	await expect(guide(page)).toHaveCount(0);
	await expect(opener(page)).toBeFocused();

	await expect(page.getByRole('button', { name: 'Pontos' })).toHaveAttribute(
		'aria-pressed',
		'true'
	);
	expect(await savedHelp(page)).toEqual(before);
});

test('a phone-sized window opens the guide too, without sideways scroll (GER-11, GER-10)', async ({
	page
}) => {
	await page.setViewportSize({ width: 390, height: 844 });
	await page.goto('./');
	await opener(page).click();
	await expect(guide(page)).toBeVisible();
	const box = (await guide(page).boundingBox())!;
	expect(box.x).toBeGreaterThanOrEqual(0);
	expect(box.x + box.width).toBeLessThanOrEqual(390);
	await guide(page).getByRole('button', { name: 'Fechar' }).click();
	await expect(opener(page)).toBeFocused();
});

test('in English, the guide follows the language (GER-11, IDI)', async ({ page }) => {
	await page.goto('./');
	await page.getByRole('button', { name: 'English' }).click();
	await page.getByRole('button', { name: 'How to use' }).click();
	const en = page.getByRole('dialog', { name: 'How to use dotree' });
	for (const title of ['Steps', 'Keyboard', 'Help', 'Limitations', 'References'])
		await expect(en.getByRole('heading', { name: title })).toBeVisible();
});
