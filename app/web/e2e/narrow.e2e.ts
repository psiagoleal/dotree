// Caminho relativo: app/web/e2e/narrow.e2e.ts
// MT-16, GER-10: below 768 px the exercise is read-only, with a notice; matrix, diagram,
// verification and VHDL stay available, and the page never scrolls sideways.
import { expect, test, type Page } from '@playwright/test';
import { r2ByMouse } from './r2';

test.use({ locale: 'pt-BR' });

const PHONE = { width: 390, height: 844 };
const notice = (page: Page) => page.getByTestId('read-only');

async function noSidewaysScroll(page: Page) {
	const overflow = await page.evaluate(
		() => document.documentElement.scrollWidth - document.documentElement.clientWidth
	);
	expect(overflow).toBeLessThanOrEqual(0);
}

test('a phone-sized window shows the exercise read-only, with a notice (GER-10)', async ({
	page
}) => {
	await page.setViewportSize(PHONE);
	await page.goto('./');
	await page.getByRole('button', { name: 'Soma de produtos' }).click();
	await expect(notice(page)).toContainText('modo leitura');
	await expect(page.getByTestId('level-current').locator('.dot').first()).toBeVisible();
	await expect(page.getByTestId('level-current').getByRole('button')).toHaveCount(0);
	for (const name of ['Formar célula', 'Concluir nível', 'Fechar com CPA', 'Desfazer']) {
		await expect(page.getByRole('button', { name })).toHaveCount(0);
	}
	await noSidewaysScroll(page);
});

test('narrowing keeps the work, the diagram, verification and VHDL; widening gives the actions back (GER-10)', async ({
	page
}) => {
	await page.goto('./');
	await r2ByMouse(page);
	await page.getByRole('button', { name: 'Fechar com CPA' }).click();
	await expect(page.getByTestId('complete')).toBeVisible();

	await page.setViewportSize(PHONE);
	await expect(notice(page)).toBeVisible();
	await expect(page.getByRole('button', { name: 'Desfazer' })).toHaveCount(0);
	await expect(page.getByRole('button', { name: 'Voltar a este nível' })).toHaveCount(0);
	// Shortcuts do not change the circuit either.
	await page.locator('body').press('Control+z');
	await expect(page.getByTestId('complete')).toBeVisible();

	await expect(page.getByTestId('diagram')).toBeVisible();
	await expect(page.getByTestId('wiring-table')).toBeAttached();
	await page.getByRole('button', { name: 'Verificar circuito' }).click();
	await expect(page.getByTestId('verify-result')).toHaveText('16.384 casos, exaustivo, 0 erros.');
	await expect(page.getByRole('button', { name: 'Baixar todos (.zip)' })).toBeEnabled();
	await noSidewaysScroll(page);

	await page.setViewportSize({ width: 1024, height: 800 });
	await expect(notice(page)).toHaveCount(0);
	await expect(page.getByRole('button', { name: 'Desfazer' })).toBeEnabled();
});
