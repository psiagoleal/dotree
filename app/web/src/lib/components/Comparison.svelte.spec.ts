// Caminho relativo: app/web/src/lib/components/Comparison.svelte.spec.ts
// MT-38: the comparison on R1 (spec 00), against the real core: the figures of each
// solution, computed by hand from their metrics, and nothing shown before it was revealed.
import { beforeAll, describe, expect, it } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import Comparison from './Comparison.svelte';
import { figures } from '#lib/comparison.ts';
import { loadCore, type CompressionSession, type Core, type Method } from '#lib/core.ts';
import { R1, r2, r2Levels } from '#lib/testing/r2.ts';

let core: Core;

beforeAll(async () => {
	core = await loadCore();
});

function solutionSnap(method: Method) {
	const s = core.openSession(R1.expression, R1.widths);
	const sol = s.solution(false, method);
	try {
		return sol.snapshot();
	} finally {
		sol.free();
		s.free();
	}
}

async function show(session: CompressionSession) {
	return render(Comparison, { session, snap: session.snapshot(), helps: session.helpLog() });
}

function cells(container: HTMLElement, testid: string): string[][] {
	return [...container.querySelectorAll(`[data-testid="${testid}"] tr`)].map((r) =>
		[...r.querySelectorAll('th, td')].map((c) => c.textContent?.trim() ?? '')
	);
}

describe('figures (MT-38)', () => {
	it('R1: the student (R2), Dadda and Wallace, as computed by hand from their metrics', () => {
		expect(figures(r2(core).snapshot())).toEqual({
			complete: true,
			area: 15,
			delay: 6,
			fa: 13,
			ha: 4,
			andGates: 13,
			levels: 3,
			cpaWidth: 6,
			critical: 12,
			areaDelay: 90,
			areaDelay2: 540
		});
		expect(figures(solutionSnap('dadda'))).toMatchObject({
			complete: true,
			area: 17,
			delay: 7,
			fa: 13,
			ha: 8,
			andGates: 13,
			levels: 4,
			cpaWidth: 6,
			areaDelay: 119,
			areaDelay2: 833
		});
		expect(figures(solutionSnap('wallace'))).toMatchObject({
			complete: true,
			area: 14.5,
			delay: 6,
			fa: 13,
			ha: 3,
			andGates: 13,
			levels: 4,
			cpaWidth: 3,
			areaDelay: 87,
			areaDelay2: 522
		});
	});

	it('an incomplete circuit counts its concluded tree levels and has no CPA yet', () => {
		expect(figures(r2Levels(core, 2).snapshot())).toMatchObject({
			complete: false,
			levels: 2,
			cpaWidth: null
		});
	});
});

describe('Comparison (MT-38, GER-01)', () => {
	it('shows nothing of a solution before it was revealed, and counts nothing', async () => {
		const session = r2(core);
		const log = session.helpLog();
		const { container } = await show(session);
		await expect.element(page.getByTestId('comparison-empty')).toBeVisible();
		expect(container.querySelector('table')).toBeNull();
		expect(session.helpLog()).toEqual(log);
	});

	it('R1: the student, Dadda and Wallace side by side; only Wallace is non-dominated', async () => {
		const session = r2(core);
		session.solution(false, 'dadda').free();
		session.solution(false, 'wallace').free();
		const log = session.helpLog();
		const { container } = await show(session);
		expect(cells(container, 'comparison-table')).toEqual([
			['Medida', 'Seu circuito', 'Dadda (clássico)', 'Wallace'],
			['Área (A_FA)', '15', '17', '14,5'],
			['Atraso (T_FA)', '6', '7', '6'],
			['FA', '13', '13', '13'],
			['HA', '4', '8', '3'],
			['AND', '13', '13', '13'],
			['Níveis antes do CPA', '3', '4', '4'],
			['Largura do CPA (bits)', '6', '6', '3'],
			['Células no caminho crítico', '12', expect.any(String), expect.any(String)],
			['Área × atraso', '90', '119', '87'],
			['Área × atraso²', '540', '833', '522'],
			['Não dominada', 'não', 'não', 'sim']
		]);
		await expect.element(page.getByText(/^Dadda clássico: HA quando/)).toBeVisible();
		// Only step 1 was revealed: no height table of a solution yet.
		expect(container.querySelector('[data-testid^="height-table-"]')).toBeNull();
		expect(session.helpLog()).toEqual(log);
	});

	it('a solution whose levels were revealed brings its height table; the other is missing', async () => {
		const session = r2Levels(core, 2);
		session.solution(false, 'wallace').free();
		session.solution(true, 'wallace').free();
		const { container } = await show(session);
		expect(cells(container, 'comparison-table')[0]).toEqual([
			'Medida',
			'Seu circuito (incompleto)',
			'Wallace'
		]);
		expect(cells(container, 'comparison-table').at(-1)).toEqual(['Não dominada', '—', 'sim']);
		await expect
			.element(
				page.getByText('A solução de Dadda entra aqui depois que você pedir as métricas dela.')
			)
			.toBeVisible();
		expect(cells(container, 'height-table-wallace').slice(1)).toEqual([
			['0', '0', '0', '1', '2', '5', '7', '5', '4', '0'],
			['1', '0', '0', '1', '3', '5', '4', '3', '4', '0'],
			['2', '0', '0', '2', '2', '4', '3', '1', '2', '0'],
			['3', '0', '0', '2', '3', '3', '1', '1', '2', '1'],
			['4 (CPA)', '0', '1', '2', '2', '1', '1', '1', '1', '2'],
			['Resultado', '1', '1', '1', '1', '1', '1', '1', '—', '—']
		]);
		await expect
			.element(
				page.getByRole('button', { name: 'Baixar alturas da solução de Wallace (Markdown)' })
			)
			.toBeVisible();
	});
});
