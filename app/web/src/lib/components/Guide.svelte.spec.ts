// Caminho relativo: app/web/src/lib/components/Guide.svelte.spec.ts
import { describe, expect, it } from 'vitest';
import { page, userEvent } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import Guide from './Guide.svelte';

describe('Guide (GER-11)', () => {
	it('opens without a confirmation and shows the five parts', async () => {
		await render(Guide, {});
		await page.getByRole('button', { name: 'Como usar' }).click();
		const dialog = page.getByRole('dialog', { name: 'Como usar o dotree' });
		await expect.element(dialog).toBeVisible();
		for (const title of ['Passos', 'Teclado', 'Ajudas', 'Limitações', 'Referências'])
			await expect.element(dialog.getByRole('heading', { name: title })).toBeVisible();
		await expect.element(dialog.getByText(/vou tentar mais um pouco/)).toBeInTheDocument();
		await expect.element(page.getByRole('button', { name: 'Mostrar' })).not.toBeInTheDocument();
	});

	it('shows each shortcut key as a key', async () => {
		await render(Guide, {});
		await page.getByRole('button', { name: 'Como usar' }).click();
		const kbd = document.querySelectorAll('dialog kbd');
		expect([...kbd].map((k) => k.textContent)).toEqual(
			expect.arrayContaining(['Tab', 'Espaço', 'F', 'Esc', 'Ctrl+Z', 'Ctrl+Y', 'V'])
		);
	});

	it('closes by its button and gives the focus back to the opener', async () => {
		await render(Guide, {});
		const opener = page.getByRole('button', { name: 'Como usar' });
		await opener.click();
		await page.getByRole('button', { name: 'Fechar' }).click();
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
		await expect.element(opener).toHaveFocus();
	});

	it('gives the focus to the opener even when a click did not focus it (Safari)', async () => {
		await render(Guide, {});
		const opener = page.getByRole('button', { name: 'Como usar' });
		(document.activeElement as HTMLElement | null)?.blur();
		(opener.element() as HTMLButtonElement).click();
		await expect.element(page.getByRole('dialog')).toBeVisible();
		await page.getByRole('button', { name: 'Fechar' }).click();
		await expect.element(opener).toHaveFocus();
	});

	it('closes by Escape and gives the focus back to the opener', async () => {
		await render(Guide, {});
		const opener = page.getByRole('button', { name: 'Como usar' });
		await opener.click();
		await expect.element(page.getByRole('dialog')).toBeVisible();
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
		await expect.element(opener).toHaveFocus();
	});
});
