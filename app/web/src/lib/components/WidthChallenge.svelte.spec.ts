// Caminho relativo: app/web/src/lib/components/WidthChallenge.svelte.spec.ts
import { describe, expect, it, vi } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import WidthChallenge from './WidthChallenge.svelte';
import type { Matrix } from '#lib/core.ts';

// R1 as the core returns it; the dots do not matter here.
const r1: Matrix = {
	terms: [
		{ coefficient: '1', vars: 'AB', max: '49' },
		{ coefficient: '1', vars: 'CD', max: '9' },
		{ coefficient: '3', vars: 'E', max: '9' },
		{ coefficient: '3', vars: 'F', max: '3' },
		{ coefficient: '1', vars: 'G', max: '1' }
	],
	inputs: [
		{ var: 'A', width: 3, max: '7' },
		{ var: 'B', width: 3, max: '7' },
		{ var: 'C', width: 2, max: '3' },
		{ var: 'D', width: 2, max: '3' },
		{ var: 'E', width: 2, max: '3' },
		{ var: 'F', width: 1, max: '1' },
		{ var: 'G', width: 1, max: '1' }
	],
	dots: [],
	heights: [5, 7, 5, 2, 1, 0, 0],
	outputWidth: 7,
	vMax: '71',
	andGates: 13,
	warnings: []
};

const result = () => page.getByTestId('width-result');
const path = () => page.getByTestId('width-path');
const keepTrying = () => page.getByRole('button', { name: 'Vou tentar mais um pouco' });
const confirm = () => page.getByRole('button', { name: 'Mostrar', exact: true });

async function answer(value: string) {
	await page.getByLabelText('Bits da saída').fill(value);
	await page.getByRole('button', { name: 'Responder' }).click();
}

describe('WidthChallenge (EXP-09, EXP-10, GER-01, GER-03)', () => {
	it('asks without revealing W, v_max or the working', async () => {
		const onreveal = vi.fn();
		const { container } = await render(WidthChallenge, { matrix: r1, onreveal });
		await expect.element(page.getByLabelText('Bits da saída')).toBeVisible();
		expect(container.textContent).not.toMatch(/\b7\b|71|72/);
		await expect.element(result()).not.toBeInTheDocument();
		await expect.element(path()).not.toBeInTheDocument();
		expect(onreveal).not.toHaveBeenCalled();
	});

	it('a right answer reveals W (step 1) but not the working', async () => {
		const onreveal = vi.fn();
		await render(WidthChallenge, { matrix: r1, onreveal });
		await answer('7');
		await expect.element(result()).toHaveTextContent('Acertou! A saída tem W = 7 bits.');
		await expect.element(path()).not.toBeInTheDocument();
		await expect.element(page.getByLabelText('Bits da saída')).not.toBeInTheDocument();
		expect(onreveal).toHaveBeenCalledOnce();
	});

	it('a wrong answer says so and still reveals W (EXP-10)', async () => {
		await render(WidthChallenge, { matrix: r1, onreveal: () => {} });
		await answer('6');
		await expect
			.element(result())
			.toHaveTextContent('Não foi dessa vez: você respondeu 6. A saída tem W = 7 bits.');
	});

	it('refuses an answer that is not a whole number of bits, revealing nothing', async () => {
		const onreveal = vi.fn();
		await render(WidthChallenge, { matrix: r1, onreveal });
		for (const bad of ['0', '2.5', '']) {
			await answer(bad);
			await expect.element(page.getByTestId('width-invalid')).toBeVisible();
		}
		await expect.element(result()).not.toBeInTheDocument();
		expect(onreveal).not.toHaveBeenCalled();
	});

	it('giving up asks first; only confirming shows W', async () => {
		const onreveal = vi.fn();
		await render(WidthChallenge, { matrix: r1, onreveal });
		await page.getByRole('button', { name: 'Mostrar a resposta' }).click();
		await expect.element(keepTrying()).toHaveFocus();
		await keepTrying().click();
		await expect.element(result()).not.toBeInTheDocument();
		expect(onreveal).not.toHaveBeenCalled();

		await page.getByRole('button', { name: 'Mostrar a resposta' }).click();
		await confirm().click();
		await expect.element(result()).toHaveTextContent('A saída tem W = 7 bits.');
		await expect.element(path()).not.toBeInTheDocument();
		expect(onreveal).toHaveBeenCalledOnce();
	});

	it('the working is step 2, with its own request and confirmation (GER-03)', async () => {
		await render(WidthChallenge, { matrix: r1, onreveal: () => {} });
		await answer('7');
		await page.getByRole('button', { name: 'Mostrar a conta' }).click();
		await expect.element(keepTrying()).toHaveFocus();
		await keepTrying().click();
		await expect.element(path()).not.toBeInTheDocument();

		await page.getByRole('button', { name: 'Mostrar a conta' }).click();
		await confirm().click();
		const p = path().element().textContent!.replace(/\s+/g, ' ');
		expect(p).toContain('A = 23 − 1 = 7'); // 2^3 − 1, with the exponent in <sup>
		expect(p).toContain('F = 21 − 1 = 1');
		expect(p).toContain('AB + CD + 3E + 3F + G');
		expect(p).toContain('7·7 + 3·3 + 3·3 + 3·1 + 1');
		expect(p).toContain('49 + 9 + 9 + 3 + 1');
		expect(p).toContain('⌈log2 72⌉ = 7');
		expect(p).toContain('64 ≤ 71 < 128');
		await expect
			.element(page.getByRole('button', { name: 'Mostrar a conta' }))
			.not.toBeInTheDocument();
	});
});
