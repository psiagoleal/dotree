// Caminho relativo: app/web/src/lib/components/Coach.svelte.spec.ts
// Hint and next move on R1 (spec 00), against the real core.
import { beforeAll, describe, expect, it, vi } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import Coach from './Coach.svelte';
import { loadCore, type Core } from '#lib/core.ts';
import { EXAMPLES } from '#lib/expression.ts';

let core: Core;
const r1 = EXAMPLES.find((e) => e.id === 'r1')!;

beforeAll(async () => {
	core = await loadCore();
});

async function open() {
	const session = core.openSession(r1.expression, r1.widths);
	const spies = { onsuggest: vi.fn(), onform: vi.fn(), onhelp: vi.fn() };
	const points = session.snapshot().current.points;
	await render(Coach, { session, points, ...spies });
	return { session, points, ...spies };
}

const keepTrying = () => page.getByRole('button', { name: 'Vou tentar mais um pouco' });
const show = () => page.getByRole('dialog').getByRole('button', { name: 'Mostrar' });

describe('Coach (AJU-02, AJU-04, AJU-08)', () => {
	it('a hint shows only after confirmation; each request asks again and shows the next', async () => {
		const { onhelp } = await open();
		await expect.element(page.getByTestId('hint')).not.toBeInTheDocument();

		await page.getByRole('button', { name: 'Dica' }).click();
		await expect.element(keepTrying()).toHaveFocus();
		await keepTrying().click();
		await expect.element(page.getByTestId('hint')).not.toBeInTheDocument();
		expect(onhelp).not.toHaveBeenCalled();

		await page.getByRole('button', { name: 'Dica' }).click();
		await show().click();
		await expect
			.element(page.getByTestId('hint'))
			.toHaveTextContent('Com altura máxima 7, faltam no mínimo 4 níveis até o CPA.');
		expect(onhelp).toHaveBeenCalledTimes(1);

		await page.getByRole('button', { name: 'Dica' }).click();
		await expect.element(keepTrying()).toHaveFocus();
		await show().click();
		await expect
			.element(page.getByTestId('hint'))
			.toHaveTextContent(/^A coluna 2\^1 é a mais alta/);
	});

	it('the next move highlights a cell without forming it; forming takes another click', async () => {
		const { points, onsuggest, onform, onhelp } = await open();
		await page.getByRole('button', { name: 'Próxima jogada' }).click();
		await show().click();
		await expect
			.element(page.getByTestId('next-move'))
			.toHaveTextContent(/um HA com a0_b1 e a1_b0, na coluna 2\^1/);
		const ids = ['a0_b1', 'a1_b0'].map((s) => points.find((p) => p.signal === s)!.id);
		expect(onsuggest).toHaveBeenCalledWith(ids);
		expect(onform).not.toHaveBeenCalled();
		expect(onhelp).toHaveBeenCalledTimes(1);

		await page.getByRole('button', { name: 'Formar esta célula' }).click();
		expect(onform).toHaveBeenCalledWith(ids);
	});
});
