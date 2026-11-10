// Caminho relativo: app/web/src/lib/components/ProfileStatus.svelte.spec.ts
// The output profile of a height profile exercise (CMP-19) and the block name as a counted
// help (AJU-12), against the real core.
import { beforeAll, describe, expect, it, vi } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import ProfileStatus from './ProfileStatus.svelte';
import SolutionHelp from './SolutionHelp.svelte';
import Comparison from './Comparison.svelte';
import { loadCore, type CompressionSession, type Core } from '#lib/core.ts';

let core: Core;

beforeAll(async () => {
	core = await loadCore();
});

const show = () => page.getByRole('dialog').getByRole('button', { name: 'Mostrar' });

async function status(session: CompressionSession) {
	const onhelp = vi.fn();
	const snap = session.snapshot();
	await render(ProfileStatus, { session, snap, profile: snap.profile!, onhelp });
	return onhelp;
}

describe('ProfileStatus (CMP-19, AJU-12)', () => {
	it('says what was asked and which columns are still above it', async () => {
		const session = core.openSession('{3,3,3; 1,2,2,1}', {});
		await status(session);
		await expect
			.element(page.getByText(/^Pedido: \{1, 2, 2, 1\}, da coluna mais significativa/))
			.toBeVisible();
		await expect
			.element(page.getByTestId('profile-over'))
			.toHaveTextContent(
				'Ainda não: a coluna 2^2 tem 3, o perfil pede 2; a coluna 2^1 tem 3, o perfil pede 2; a coluna 2^0 tem 3, o perfil pede 1.'
			);
	});

	it('reached: area and delay of the circuit, h) of the reference set', async () => {
		const session = core.openSession('{3,3,3; 1,2,2,1}', {});
		for (const column of [0, 1, 2]) {
			const ids = session
				.snapshot()
				.current.points.filter((p) => p.column === column)
				.map((p) => p.id);
			session.formCell(ids);
		}
		session.concludeLevel();
		await status(session);
		await expect
			.element(page.getByTestId('profile-reached'))
			.toHaveTextContent('Perfil atingido: área 3 A_FA, atraso 1 T_FA.');
	});

	it('a pending cell leaves the level open', async () => {
		// {2; 2} is met by its input; an HA formed over it is not concluded yet.
		const session = core.openSession('{2; 2}', {});
		session.formCell(session.snapshot().current.points.map((p) => p.id));
		await status(session);
		await expect
			.element(page.getByTestId('profile-pending'))
			.toHaveTextContent('Conclua o nível para conferir o perfil.');
	});

	it('the block name only after confirmation, counted once and shown again uncounted', async () => {
		const session = core.openSession('{2,2,3; 1,1,1,1}', {});
		const onhelp = await status(session);
		const ask = page.getByRole('button', { name: 'Qual bloco é este?' });
		await ask.click();
		await page.getByRole('button', { name: 'Vou tentar mais um pouco' }).click();
		await expect.element(page.getByTestId('profile-block')).not.toBeInTheDocument();
		expect(session.helpLog().block).toBe(0);

		await ask.click();
		await show().click();
		await expect
			.element(page.getByTestId('profile-block'))
			.toHaveTextContent('É um somador de 3 bits com vai-um de entrada.');
		await expect.element(ask).not.toBeInTheDocument();
		expect(session.helpLog().block).toBe(1);
		expect(onhelp).toHaveBeenCalledOnce();
	});

	it('a reopened exercise shows the block it revealed, uncounted', async () => {
		const session = core.openSession('{7; 1,1,1}', {});
		session.block();
		await status(session);
		await expect
			.element(page.getByTestId('profile-block'))
			.toHaveTextContent(/^É um contador paralelo generalizado/);
		expect(session.helpLog().block).toBe(1);
	});
});

describe('Optimal solution (AJU-12)', () => {
	it('comes with the other methods only for a profile, in two steps', async () => {
		const session = core.openSession('{5,5; 1,1,1,1}', {});
		await render(SolutionHelp, { session, view: 'dots', onhelp: vi.fn(), onapply: vi.fn() });
		await page.getByRole('button', { name: 'Solução ótima', exact: true }).click();
		await show().click();
		// d) of the reference set: 6 A_FA and 4 T_FA.
		await expect
			.element(page.getByTestId('optimal-summary'))
			.toHaveTextContent(/^Solução ótima: \d+ níveis; 6 FA, 0 HA; área 6 A_FA; atraso 4 T_FA\.$/);
		await page.getByRole('button', { name: 'Ver a solução ótima nível a nível' }).click();
		await show().click();
		await expect
			.element(page.getByTestId('optimal-rule'))
			.toHaveTextContent(/^A solução ótima vem de uma busca/);
		await expect.element(page.getByTestId('optimal-result')).toBeVisible();
		await expect.element(page.getByRole('button', { name: 'Usar a solução ótima' })).toBeVisible();
		const log = session.helpLog();
		expect([log.optimal, log.optimalPath, log.solution]).toEqual([1, 1, 0]);
	});

	it('a redundant output still shows its levels: the optimum stops at the profile', async () => {
		const session = core.openSession('{5; 2,1}', {});
		session.solution(false, 'optimal').free();
		session.solution(true, 'optimal').free();
		await render(SolutionHelp, { session, view: 'dots', onhelp: vi.fn(), onapply: vi.fn() });
		await expect.element(page.getByTestId('optimal-level-0')).toBeVisible();
		await expect
			.element(page.getByTestId('optimal-result'))
			.toHaveAccessibleName(/saída no perfil/);
	});

	it('a profile too large for the search says so instead of offering it', async () => {
		const session = core.openSession('{6,7,7,6; 1,1,1,1,1,1,1}', {});
		await render(SolutionHelp, { session, view: 'dots', onhelp: vi.fn(), onapply: vi.fn() });
		await expect
			.element(page.getByTestId('optimal-unavailable'))
			.toHaveTextContent(/^Este perfil é grande demais/);
		await expect
			.element(page.getByRole('button', { name: 'Solução ótima', exact: true }))
			.not.toBeInTheDocument();
	});

	it('is not offered for an expression', async () => {
		const session = core.openSession('A*B', { A: 2, B: 2 });
		await render(SolutionHelp, { session, view: 'dots', onhelp: vi.fn(), onapply: vi.fn() });
		await expect.element(page.getByRole('button', { name: 'Solução de Wallace' })).toBeVisible();
		await expect
			.element(page.getByRole('button', { name: 'Solução ótima', exact: true }))
			.not.toBeInTheDocument();
	});

	it('enters the comparison; reaching the profile counts as complete', async () => {
		const session = core.openSession('{5; 2,1}', {});
		session.solution(false, 'optimal').free();
		const { container } = await render(Comparison, {
			session,
			snap: session.snapshot(),
			helps: session.helpLog()
		});
		const rows = [...container.querySelectorAll('[data-testid="comparison-table"] tr')].map((r) =>
			[...r.querySelectorAll('th, td')].map((c) => c.textContent?.trim())
		);
		expect(rows[0]).toEqual(['Medida', 'Seu circuito (incompleto)', 'Ótima (perfil)']);
		expect(rows[1]).toEqual(['Área (A_FA)', '0', '2']);
		expect(rows.at(-1)).toEqual(['Não dominada', '—', 'sim']);
		await expect
			.element(
				page.getByText('Com um perfil de saída, conta como completa a solução que atinge o perfil.')
			)
			.toBeVisible();
	});
});
