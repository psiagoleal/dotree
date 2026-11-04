// Caminho relativo: app/web/e2e/r2.ts
// Shared e2e steps: points of the current level and the R2 tree of spec 00 by mouse.
import type { Page } from '@playwright/test';

export const current = (page: Page) => page.getByTestId('level-current');

export const point = (page: Page, column: number, signal: string) =>
	current(page)
		.getByRole('group', { name: new RegExp(`2\\^${column},`) })
		.getByRole('button', { name: signal, exact: true });

export async function formByMouse(page: Page, column: number, signals: string[]) {
	for (const s of signals) await point(page, column, s).click();
	await page.getByRole('button', { name: 'Formar célula' }).click();
}

/** Levels 0 to 2 of the R2 tree by mouse, leaving level 3 ready for the CPA. */
export async function r2ByMouse(page: Page) {
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
