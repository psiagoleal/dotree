// Caminho relativo: app/web/src/lib/components/CircuitView.svelte.spec.ts
// Diagram, wiring table and exports of R2 (spec 05), against the real core.
import { beforeAll, describe, expect, it } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import CircuitView from './CircuitView.svelte';
import { loadCore, type CompressionSession, type Core } from '#lib/core.ts';
import { R1, r2, r2Levels } from '#lib/testing/r2.ts';

let core: Core;

beforeAll(async () => {
	core = await loadCore();
});

async function show(session: CompressionSession, critical = false) {
	return render(CircuitView, { session, snap: session.snapshot(), top: 6, critical });
}

const blocks = (c: HTMLElement) => c.querySelectorAll('svg g.block');
const svgButton = () => page.getByRole('button', { name: 'Baixar diagrama (SVG)' });
const mdButton = () => page.getByRole('button', { name: 'Baixar tabela de ligações (Markdown)' });

describe('CircuitView (DIA-01, DIA-02, DIA-04, DIA-05, DIA-07)', () => {
	it('before any concluded level there is no diagram yet (DIA-01)', async () => {
		const { container } = await show(core.openSession(R1.expression, R1.widths));
		await expect
			.element(page.getByText('O diagrama aparece quando houver um nível concluído.'))
			.toBeVisible();
		expect(container.querySelector('svg')).toBeNull();
	});

	it('an incomplete circuit is drawn up to the last concluded level; exports wait (DIA-07)', async () => {
		const { container } = await show(r2Levels(core, 2));
		expect(blocks(container)).toHaveLength(7);
		await expect
			.element(page.getByRole('status'))
			.toHaveTextContent(/^Circuito incompleto: o diagrama vai até o nível 1\./);
		await expect.element(svgButton()).toBeDisabled();
		await expect.element(mdButton()).toBeDisabled();
		expect(container.querySelectorAll('tbody tr')).toHaveLength(7);
	});

	it('R2 complete: 17 blocks, the table of spec 05 and exports enabled (DIA-04)', async () => {
		const { container } = await show(r2(core));
		expect(blocks(container)).toHaveLength(17);
		await expect.element(svgButton()).toBeEnabled();
		await expect.element(mdButton()).toBeEnabled();
		const rows = [...container.querySelectorAll('tbody tr')].map((r) =>
			[...r.querySelectorAll('td')].map((td) => td.textContent?.trim())
		);
		expect(rows).toHaveLength(17);
		expect(rows[2]).toEqual(['HA3', '0', '22', 'a0_b2', 'a1_b1', '—', 's3', 'k3']);
		expect(rows[16]).toEqual(['HA17', 'CPA', '25', 'k11', 'k16', '—', 's17 → R(5)', 'k17 → R(6)']);
		await expect.element(page.getByRole('img', { name: 'Diagrama de blocos' })).toBeVisible();
	});

	it('the AND block expands into its 13 gates on request (DIA-02)', async () => {
		const { container } = await show(r2(core));
		expect(container.querySelectorAll('svg g.gate')).toHaveLength(0);
		await page.getByRole('button', { name: 'Expandir portas AND' }).click();
		expect(container.querySelectorAll('svg g.gate')).toHaveLength(13);
	});

	it('highlights the critical cells when asked (DIA-05)', async () => {
		const { container, rerender } = await show(r2(core));
		expect(container.querySelectorAll('svg g.block.critical')).toHaveLength(0);
		await rerender({ critical: true });
		const cells = [...container.querySelectorAll('svg g.block.critical')].map((g) =>
			Number(g.getAttribute('data-cell'))
		);
		expect(cells.sort((a, b) => a - b)).toEqual([1, 2, 4, 6, 8, 9, 12, 13, 14, 15, 16, 17]);
	});
});
