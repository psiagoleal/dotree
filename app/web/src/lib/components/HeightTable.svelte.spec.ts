// Caminho relativo: app/web/src/lib/components/HeightTable.svelte.spec.ts
// The height table of R2 (spec 02, CMP-16), against the real core.
import { beforeAll, describe, expect, it } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import HeightTable from './HeightTable.svelte';
import { loadCore, type CompressionSession, type Core } from '#lib/core.ts';
import { R1, formOn, r2 } from '#lib/testing/r2.ts';

let core: Core;

beforeAll(async () => {
	core = await loadCore();
});

async function show(session: CompressionSession) {
	return render(HeightTable, {
		rows: session.heightTable(),
		markdown: (h) => session.heightTableMarkdown(h)
	});
}

function cells(container: HTMLElement): string[][] {
	return [...container.querySelectorAll('tr')].map((r) =>
		[...r.querySelectorAll('th, td')].map((c) => c.textContent?.trim() ?? '')
	);
}

const exportButton = () =>
	page.getByRole('button', { name: 'Baixar tabela de alturas (Markdown)' });

describe('HeightTable (CMP-16)', () => {
	it('R2 complete: one row per level, highest column first, and the result', async () => {
		const { container } = await show(r2(core));
		await expect.element(page.getByRole('table', { name: 'Tabela de alturas' })).toBeVisible();
		expect(cells(container)).toEqual([
			['Nível', '26', '25', '24', '23', '22', '21', '20', 'FA', 'HA'],
			['0', '0', '0', '1', '2', '5', '7', '5', '2', '1'],
			['1', '0', '0', '1', '3', '6', '3', '5', '4', '0'],
			['2', '0', '0', '2', '3', '3', '1', '5', '3', '1'],
			['3 (CPA)', '0', '1', '2', '2', '1', '2', '3', '4', '2'],
			['Resultado', '1', '1', '1', '1', '1', '1', '1', '—', '—']
		]);
		await expect.element(exportButton()).toBeEnabled();
	});

	it('from level 0, the current level counts the cells formed on it; export at any time', async () => {
		const session = core.openSession(R1.expression, R1.widths);
		formOn(session, 1, ['a0_b1', 'a1_b0', 'c0_d1']);
		const { container } = await show(session);
		expect(cells(container).slice(1)).toEqual([
			['0 (atual)', '0', '0', '1', '2', '5', '7', '5', '1', '0']
		]);
		await expect.element(exportButton()).toBeEnabled();
	});
});
