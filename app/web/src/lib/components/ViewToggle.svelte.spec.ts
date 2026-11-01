// Caminho relativo: app/web/src/lib/components/ViewToggle.svelte.spec.ts
import { describe, expect, it, vi } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import ViewToggle from './ViewToggle.svelte';

describe('ViewToggle (VIS-01, VIS-03)', () => {
	it('marks the current view and reports the other one when chosen', async () => {
		const onchange = vi.fn();
		await render(ViewToggle, { mode: 'dots', onchange });
		const dots = page.getByRole('button', { name: 'Pontos' });
		const indexed = page.getByRole('button', { name: 'Variáveis indexadas' });
		await expect.element(dots).toHaveAttribute('aria-pressed', 'true');
		await expect.element(indexed).toHaveAttribute('aria-pressed', 'false');
		await expect.element(indexed).toHaveAttribute('aria-keyshortcuts', 'V');
		await indexed.click();
		expect(onchange).toHaveBeenCalledWith('indexed');
	});
});
