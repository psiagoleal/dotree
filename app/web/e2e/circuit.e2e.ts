// Caminho relativo: app/web/e2e/circuit.e2e.ts
// MT-14a: diagram, wiring table and their exports (spec 05).
import { readFileSync } from 'node:fs';
import { expect, test } from '@playwright/test';
import { r2ByMouse } from './r2';

test.use({ locale: 'pt-BR' });

/** The reference table of spec 05, the same text the core test reads. */
function referenceTable(): string {
	const spec = readFileSync(new URL('../../../docs/specs/05-diagrama.md', import.meta.url), 'utf8');
	const start = spec.indexOf('| Célula | Nível |');
	return spec
		.slice(start)
		.split('\n')
		.filter((l, i, all) => all.slice(0, i + 1).every((x) => x.startsWith('|')))
		.join('\n');
}

test('no diagram before a concluded level (DIA-01)', async ({ page }) => {
	await page.goto('./');
	await page.getByRole('button', { name: 'Soma de produtos' }).click();
	await expect(
		page.getByText('O diagrama aparece quando houver um nível concluído.')
	).toBeVisible();
	await page.getByRole('button', { name: 'Multiplicador 4×4' }).click();
	const svg = page.getByRole('button', { name: 'Baixar diagrama (SVG)' });
	await expect(svg).toHaveCount(0);
});

test('R2: the diagram and the wiring table download as SVG and Markdown (DIA-02, DIA-06)', async ({
	page
}) => {
	await page.goto('./');
	await r2ByMouse(page);
	const svgButton = page.getByRole('button', { name: 'Baixar diagrama (SVG)' });
	await expect(svgButton).toBeDisabled();
	await expect(
		page.getByText(/^Circuito incompleto: o diagrama vai até o nível 2\./)
	).toBeVisible();
	await page.getByRole('button', { name: 'Fechar com CPA' }).click();
	await expect(svgButton).toBeEnabled();

	// DIA-02: one block, or each of the 13 gates.
	const diagram = page.getByTestId('diagram');
	await expect(diagram.locator('g.gate')).toHaveCount(0);
	await page.getByRole('button', { name: 'Expandir portas AND' }).click();
	await expect(diagram.locator('g.gate')).toHaveCount(13);

	// DIA-06: the SVG is a well-formed standalone file with the 17 cells.
	const [svgFile] = await Promise.all([page.waitForEvent('download'), svgButton.click()]);
	expect(svgFile.suggestedFilename()).toBe('dotree-diagrama.svg');
	const svg = readFileSync((await svgFile.path())!, 'utf8');
	const parsed = await page.evaluate((text) => {
		const doc = new DOMParser().parseFromString(text, 'image/svg+xml');
		return {
			error: doc.querySelector('parsererror')?.textContent ?? null,
			root: doc.documentElement.namespaceURI,
			blocks: doc.querySelectorAll('g.block').length
		};
	}, svg);
	expect(parsed).toEqual({ error: null, root: 'http://www.w3.org/2000/svg', blocks: 17 });

	// DIA-06: the Markdown table is the reference table of spec 05, character for character.
	const [mdFile] = await Promise.all([
		page.waitForEvent('download'),
		page.getByRole('button', { name: 'Baixar tabela (Markdown)' }).click()
	]);
	expect(mdFile.suggestedFilename()).toBe('dotree-ligacoes.md');
	const md = readFileSync((await mdFile.path())!, 'utf8');
	expect(md.trim()).toBe(referenceTable().trim());
});
