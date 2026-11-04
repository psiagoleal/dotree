// Caminho relativo: app/web/src/lib/diagram.spec.ts
import { describe, expect, it } from 'vitest';
import { diagramSvg, geometry, type DiagramOptions } from './diagram';
import type { Diagram } from './core';

// A small circuit: FA1 and FA2 side by side in 2^1, HA3 in 2^2, FA4 below in 2^1.
const d: Diagram = {
	complete: true,
	outputWidth: 3,
	rows: 3,
	andGates: 2,
	blocks: [
		{ cell: 1, kind: 'fa', row: 1, column: 1, slot: 0 },
		{ cell: 2, kind: 'fa', row: 1, column: 1, slot: 1 },
		{ cell: 3, kind: 'ha', row: 1, column: 2, slot: 0 },
		{ cell: 4, kind: 'fa', row: 2, column: 1, slot: 0 }
	],
	edges: [
		{ from: 'inputs', to: 1, label: 'a0_b1' },
		{ from: 'inputs', to: 1, label: 'a1_b0' },
		{ from: 'inputs', to: 1, label: 'c0' },
		{ from: 1, to: 4, label: 's1' },
		{ from: 2, to: 4, label: 's2' },
		{ from: 'inputs', to: 4, label: 'x<y' },
		{ from: 4, to: 'R1', label: 's4' },
		{ from: 3, to: 'R0', label: 's3' }
	],
	outputs: ['s3', 's4', null],
	critical: [1, 4],
	ands: [
		{ ident: 'a0_b1', factors: ['a0', 'b1'] },
		{ ident: 'a1_b0', factors: ['a1', 'b0'] }
	]
};

const o: DiagramOptions = {
	top: 2,
	critical: false,
	expandAnds: false,
	text: { title: 'Diagrama de blocos', inputs: 'Entradas e portas AND (2)' }
};

const count = (s: string, re: RegExp) => (s.match(re) ?? []).length;

describe('geometry (DIA-01)', () => {
	const g = geometry(d, o);
	const box = (n: number) => g.blocks.find((b) => b.cell === n)!;

	it('puts higher weights on the left and rows downwards', () => {
		expect(box(3).x).toBeLessThan(box(1).x);
		expect(box(4).y).toBeGreaterThan(box(1).y);
		expect(box(4).x).toBe(box(1).x);
		expect(box(1).y).toBe(box(3).y);
	});

	it('places blocks of the same row and column side by side, without overlap', () => {
		expect(box(2).y).toBe(box(1).y);
		expect(box(2).x).toBeGreaterThan(box(1).x + box(1).w);
		expect(box(3).x + box(3).w).toBeLessThan(box(1).x);
		for (const b of g.blocks) {
			expect(b.x + b.w).toBeLessThanOrEqual(g.width);
			expect(b.y + b.h).toBeLessThanOrEqual(g.height);
		}
	});
});

describe('diagramSvg (DIA-02, DIA-03, DIA-05, DIA-06)', () => {
	it('is a standalone SVG with a title and one group per block', () => {
		const svg = diagramSvg(d, o);
		expect(svg.startsWith('<svg xmlns="http://www.w3.org/2000/svg"')).toBe(true);
		expect(svg).toContain('<title>Diagrama de blocos</title>');
		expect(count(svg, /<g class="block /g)).toBe(4);
		expect(svg).toContain('>FA4</text>');
		expect(svg).toContain('>Entradas e portas AND (2)</text>');
	});

	it('labels every connection with its wire, escaped (DIA-03)', () => {
		const svg = diagramSvg(d, o);
		for (const l of ['a0_b1', 'a1_b0', 'c0', 's1', 's2', 's4', 's3']) {
			expect(svg).toContain(`>${l}</text>`);
		}
		expect(svg).toContain('>x&lt;y</text>');
		expect(svg).not.toContain('x<y');
	});

	it('names the result bits once complete; an empty one is 0', () => {
		const svg = diagramSvg(d, o);
		expect(svg).toContain('>R(0)</text>');
		expect(svg).toContain('>R(2) = 0</text>');
		expect(diagramSvg({ ...d, complete: false, outputs: [] }, o)).not.toContain('R(0)');
	});

	it('highlights the critical cells only on request (DIA-05)', () => {
		expect(count(diagramSvg(d, o), /class="block [fh]a critical"/g)).toBe(0);
		expect(count(diagramSvg(d, { ...o, critical: true }), /class="block [fh]a critical"/g)).toBe(2);
	});

	it('expands the AND block into its gates on request (DIA-02)', () => {
		expect(count(diagramSvg(d, o), /<g class="gate"/g)).toBe(0);
		const svg = diagramSvg(d, { ...o, expandAnds: true });
		expect(count(svg, /<g class="gate"/g)).toBe(2);
		expect(svg).toContain('>a1·b0</text>');
	});
});
