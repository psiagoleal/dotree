// Caminho relativo: app/web/src/lib/components/DotMatrix.svelte.spec.ts
import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import DotMatrix from './DotMatrix.svelte';
import TermLegend from './TermLegend.svelte';
import type { Matrix } from '#lib/core.ts';

// 2A + B with A of 2 bits, B of 1 bit: terms 2A (shifted) and B.
const matrix: Matrix = {
	terms: [
		{ coefficient: '2', vars: 'A' },
		{ coefficient: '1', vars: 'B' }
	],
	dots: [
		{ id: 0, column: 1, term: 0, signal: 'a0', factors: [{ var: 'A', bit: 0 }] },
		{ id: 1, column: 2, term: 0, signal: 'a1', factors: [{ var: 'A', bit: 1 }] },
		{ id: 2, column: 0, term: 1, signal: 'b0', factors: [{ var: 'B', bit: 0 }] }
	],
	heights: [1, 1, 1, 0],
	outputWidth: 4,
	vMax: '7',
	andGates: 0,
	warnings: []
};

describe('DotMatrix', () => {
	it('draws the highest weight first and hides columns above the last dot (EXP-09)', async () => {
		const { container } = await render(DotMatrix, { matrix });
		const weights = [...container.querySelectorAll('.weight')].map((w) => w.textContent);
		expect(weights).toEqual(['22', '21', '20']); // 2^2, 2^1, 2^0 — no 2^3
	});

	it('colours each dot by its term and names its signal (COR-01, VIS-04)', async () => {
		const { container } = await render(DotMatrix, { matrix });
		const dots = [...container.querySelectorAll<HTMLElement>('.dot')];
		expect(dots).toHaveLength(3);
		const a1 = dots.find((d) => d.dataset.signal === 'a1')!;
		expect(a1.style.getPropertyValue('--c')).toBe('var(--dot-term-1)');
		expect(a1.getAttribute('aria-label')).toBe('a1');
		expect(a1.getAttribute('tabindex')).toBe('0');
		const b0 = dots.find((d) => d.dataset.signal === 'b0')!;
		expect(b0.style.getPropertyValue('--c')).toBe('var(--dot-term-2)');
	});
});

describe('TermLegend', () => {
	it('lists each term once, in its colour (COR-01)', async () => {
		const { container } = await render(TermLegend, { terms: matrix.terms });
		const items = [...container.querySelectorAll('li')].map((li) =>
			li.textContent?.replace(/\s+/g, ' ').trim()
		);
		expect(items).toEqual(['1. 2A', '2. B']);
	});

	it('colours repeat from the 9th term on, and entries stay numbered (COR-03)', async () => {
		const terms = Array.from({ length: 10 }, (_, i) => ({
			coefficient: '1',
			vars: String.fromCharCode(65 + i)
		}));
		const { container } = await render(TermLegend, { terms });
		const swatches = [...container.querySelectorAll<HTMLElement>('.swatch')];
		expect(swatches[8].style.getPropertyValue('--c')).toBe('var(--dot-term-1)');
		expect(container.querySelectorAll('.index')[9].textContent).toBe('10.');
	});
});
