// Caminho relativo: app/web/src/lib/components/LanguageSwitch.svelte.spec.ts
import { describe, expect, it } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import LanguageSwitch from './LanguageSwitch.svelte';

describe('LanguageSwitch', () => {
	it('offers both languages and marks exactly one as current', async () => {
		render(LanguageSwitch);
		const pt = page.getByRole('button', { name: 'Português' });
		const en = page.getByRole('button', { name: 'English' });
		await expect.element(pt).toBeInTheDocument();
		await expect.element(en).toBeInTheDocument();
		const pressed = [pt, en].filter((b) => b.element().getAttribute('aria-pressed') === 'true');
		expect(pressed).toHaveLength(1);
	});
});
