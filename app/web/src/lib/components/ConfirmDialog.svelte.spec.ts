// Caminho relativo: app/web/src/lib/components/ConfirmDialog.svelte.spec.ts
import { describe, expect, it } from 'vitest';
import { page, userEvent } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import ConfirmDialog from './ConfirmDialog.svelte';

const props = { title: 'Ver a resposta?', body: 'Tente mais um pouco.' };

describe('ConfirmDialog (AJU-02)', () => {
	it('opens with the focus on "keep trying", and cancelling resolves false', async () => {
		const { component } = await render(ConfirmDialog, props);
		const answer = component.ask();
		const keepTrying = page.getByRole('button', { name: 'Vou tentar mais um pouco' });
		await expect.element(page.getByRole('dialog', { name: 'Ver a resposta?' })).toBeVisible();
		await expect.element(keepTrying).toHaveFocus();
		await keepTrying.click();
		await expect(answer).resolves.toBe(false);
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
	});

	it('confirming is the second option and resolves true', async () => {
		const { component } = await render(ConfirmDialog, props);
		const answer = component.ask();
		const buttons = page.getByRole('dialog').getByRole('button').elements();
		expect(buttons.map((b) => b.textContent?.trim())).toEqual([
			'Vou tentar mais um pouco',
			'Mostrar'
		]);
		await page.getByRole('button', { name: 'Mostrar' }).click();
		await expect(answer).resolves.toBe(true);
	});

	it('Escape counts as "keep trying"; each request asks again (D-5)', async () => {
		const { component } = await render(ConfirmDialog, props);
		const first = component.ask();
		await userEvent.keyboard('{Escape}');
		await expect(first).resolves.toBe(false);
		const second = component.ask();
		await expect
			.element(page.getByRole('button', { name: 'Vou tentar mais um pouco' }))
			.toHaveFocus();
		await page.getByRole('button', { name: 'Mostrar' }).click();
		await expect(second).resolves.toBe(true);
	});
});
