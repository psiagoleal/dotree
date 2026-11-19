// Caminho relativo: app/web/src/lib/components/Zoom.svelte.spec.ts
import { describe, expect, it, vi } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import Zoom from './Zoom.svelte';

const group = () => page.getByRole('group', { name: 'Zoom da matriz' });

describe('Zoom (VIS-05, DIA-08)', () => {
	it('shows the zoom and steps out and in', async () => {
		const onchange = vi.fn();
		await render(Zoom, { zoom: 1, onchange, label: 'Zoom da matriz', measure: () => null });
		await expect.element(group().getByText('100%')).toBeVisible();
		await group().getByRole('button', { name: 'Diminuir' }).click();
		expect(onchange).toHaveBeenLastCalledWith(0.75);
		await group().getByRole('button', { name: 'Aumentar' }).click();
		expect(onchange).toHaveBeenLastCalledWith(1.5);
	});

	it('disables the step that would leave the range', async () => {
		await render(Zoom, {
			zoom: 0.25,
			onchange: () => {},
			label: 'Zoom da matriz',
			measure: () => null
		});
		await expect.element(group().getByRole('button', { name: 'Diminuir' })).toBeDisabled();
		await expect.element(group().getByRole('button', { name: 'Aumentar' })).toBeEnabled();
	});

	it('fits the content to the room it has, and does nothing before drawing', async () => {
		const onchange = vi.fn();
		let size: { available: number; natural: number } | null = null;
		await render(Zoom, { zoom: 1, onchange, label: 'Zoom da matriz', measure: () => size });
		const fit = group().getByRole('button', { name: 'Ajustar à largura' });
		await fit.click();
		expect(onchange).not.toHaveBeenCalled();
		size = { available: 900, natural: 1800 };
		await fit.click();
		expect(onchange).toHaveBeenLastCalledWith(0.5);
	});
});
