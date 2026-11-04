// Caminho relativo: app/web/src/lib/components/VhdlView.svelte.spec.ts
// VHDL panel (spec 06) against the real core.
import { beforeAll, describe, expect, it, vi } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import VhdlView from './VhdlView.svelte';
import { loadCore, type Core } from '#lib/core.ts';
import { r2, r2Levels } from '#lib/testing/r2.ts';

let core: Core;

beforeAll(async () => {
	core = await loadCore();
});

const zipButton = () => page.getByRole('button', { name: 'Baixar todos (.zip)' });
const nameInput = () => page.getByRole('textbox', { name: 'Nome da entidade' });
const fileNames = (c: HTMLElement) =>
	[...c.querySelectorAll('pre[data-file]')].map((p) => p.getAttribute('data-file'));

describe('VhdlView (VHD-05, VHD-07, VHD-08)', () => {
	it('is disabled while the circuit is incomplete, saying why (VHD-07)', async () => {
		const s = r2Levels(core, 3);
		const { container } = await render(VhdlView, { session: s, snap: s.snapshot() });
		await expect.element(zipButton()).toBeDisabled();
		await expect.element(page.getByText('O VHDL exige o circuito completo.')).toBeVisible();
		expect(fileNames(container)).toEqual([]);
	});

	it('lists the files of R2 under the chosen name', async () => {
		const s = r2(core);
		const { container } = await render(VhdlView, { session: s, snap: s.snapshot() });
		expect(fileNames(container)).toEqual([
			'full_adder.vhd',
			'half_adder.vhd',
			'dotree_circuito.vhd',
			'tb_dotree_circuito.vhd'
		]);
		await nameInput().fill('meu_somador');
		await expect
			.element(page.getByRole('button', { name: 'Copiar meu_somador.vhd' }))
			.toBeVisible();
		expect(container.querySelector('pre[data-file="meu_somador.vhd"]')?.textContent).toContain(
			'entity meu_somador is'
		);
		await expect.element(zipButton()).toBeEnabled();
	});

	it('refuses a name the circuit already uses (VHD-08)', async () => {
		const s = r2(core);
		const { container } = await render(VhdlView, { session: s, snap: s.snapshot() });
		await nameInput().fill('FA1');
		await expect.element(nameInput()).toHaveAttribute('aria-invalid', 'true');
		await expect.element(page.getByText(/^Nome inválido/)).toBeVisible();
		await expect.element(zipButton()).toBeDisabled();
		expect(fileNames(container)).toEqual([]);
	});

	it('copies a file and says so (VHD-05)', async () => {
		const writeText = vi.fn(() => Promise.resolve());
		vi.spyOn(navigator, 'clipboard', 'get').mockReturnValue({ writeText } as unknown as Clipboard);
		const s = r2(core);
		const { container } = await render(VhdlView, { session: s, snap: s.snapshot() });
		await page.getByRole('button', { name: 'Copiar full_adder.vhd' }).click();
		expect(writeText).toHaveBeenCalledWith(
			container.querySelector('pre[data-file="full_adder.vhd"]')?.textContent
		);
		await expect.element(page.getByRole('status')).toHaveTextContent('full_adder.vhd copiado.');
		vi.restoreAllMocks();
	});
});
