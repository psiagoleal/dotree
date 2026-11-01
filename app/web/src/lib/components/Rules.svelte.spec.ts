// Caminho relativo: app/web/src/lib/components/Rules.svelte.spec.ts
import { describe, expect, it } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import Rules from './Rules.svelte';

describe('Rules (AJU-01)', () => {
	it('opens at any time, without a confirmation', async () => {
		await render(Rules, {});
		await page.getByText('Regras').click();
		await expect.element(page.getByText(/^FA: 3 pontos da mesma coluna/)).toBeVisible();
		await expect.element(page.getByText(/^O CPA fecha um nível/)).toBeVisible();
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
	});
});
