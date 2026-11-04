// Caminho relativo: app/web/src/lib/diagram.ts
// The block diagram as SVG (spec 05). The core gives a grid (row, column, slot); this turns it
// into pixels and markup. One string serves the page and the exported file: colours come from
// the page's CSS variables, with fixed fallbacks for the standalone file (DIA-06).
import type { Diagram, DiagramEnd } from './core';

export interface DiagramOptions {
	/** Highest column drawn, so the diagram lines up with the level panels. */
	top: number;
	/** Highlight the cells on a critical path (DIA-05). */
	critical: boolean;
	/** Draw each AND gate instead of one block (DIA-02). */
	expandAnds: boolean;
	text: {
		/** Accessible name of the drawing. */
		title: string;
		/** Caption of the input block, e.g. "Entradas e portas AND (13)". */
		inputs: string;
	};
}

export interface BlockBox {
	cell: number;
	kind: 'fa' | 'ha';
	x: number;
	y: number;
	w: number;
	h: number;
}

export interface Geometry {
	width: number;
	height: number;
	blocks: BlockBox[];
}

// Sizes in px. A block is wide enough for one wire label above each input port.
const M = 16; // margin
const BW = 96; // block width
const BH = 36; // block height
const GAP = 24; // between blocks and between columns
const RH = 112; // row pitch
const HEAD = 24; // weights 2^j under the input block
const LABEL = 30; // room for the labels above a row of blocks
const GATE_W = 72;
const GATE_H = 26;
const OUT = 56; // result row

export function geometry(d: Diagram, o: DiagramOptions): Geometry {
	return layout(d, o).g;
}

/** Where everything goes; `diagramSvg` only writes it out. */
function layout(d: Diagram, o: DiagramOptions) {
	const top = Math.max(o.top, ...d.blocks.map((b) => b.column), d.outputs.length - 1, 0);
	const slots = (c: number) =>
		Math.max(1, ...d.blocks.filter((b) => b.column === c).map((b) => b.slot + 1));
	// Left edge of each column, highest weight first.
	const colX = new Map<number, number>();
	let x = M;
	for (let c = top; c >= 0; c--) {
		colX.set(c, x);
		x += slots(c) * (BW + GAP);
	}
	const width = Math.max(x - GAP + M, 2 * M + 320);

	// Input block: one box, or one gate per partial product in a wrapping grid.
	const perLine = Math.max(1, Math.floor((width - 2 * M - 16) / (GATE_W + 8)));
	const gateLines = o.expandAnds ? Math.ceil(d.ands.length / perLine) : 0;
	const inputsH = 32 + gateLines * (GATE_H + 8);
	const gates = o.expandAnds
		? d.ands.map((a, i) => ({
				...a,
				x: M + 8 + (i % perLine) * (GATE_W + 8),
				y: M + 28 + Math.floor(i / perLine) * (GATE_H + 8)
			}))
		: [];
	const headY = M + inputsH + 18;
	const rowY = (row: number) => M + inputsH + HEAD + LABEL + (row - 1) * RH;

	const blocks: BlockBox[] = d.blocks.map((b) => ({
		cell: b.cell,
		kind: b.kind,
		x: colX.get(b.column)! + b.slot * (BW + GAP),
		y: rowY(b.row),
		w: BW,
		h: BH
	}));
	const outY = rowY(Math.max(2, d.rows)) - LABEL + 12;
	const height = (d.complete ? outY + OUT : rowY(Math.max(2, d.rows)) - LABEL) + M;
	const centre = (c: number) => colX.get(c)! + BW / 2;
	return { g: { width, height, blocks }, top, colX, centre, headY, inputsH, gates, outY };
}

const esc = (s: string) =>
	s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

const STYLE = `
.line{fill:none;stroke:var(--color-text,#1d1d1f);stroke-width:1.2}
.box{fill:var(--color-surface,#fff);stroke:var(--color-text,#1d1d1f);stroke-width:1.5}
.ha .box{stroke-dasharray:5 3}
.critical .box{stroke:var(--color-accent,#4a3fc9);stroke-width:3}
text{font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;fill:var(--color-text,#1d1d1f)}
.name{font-size:12px;font-weight:600}
.wire{font-size:9px;paint-order:stroke;stroke:var(--color-bg,#fff);stroke-width:3px}
.weight,.caption{font-size:11px;fill:var(--color-muted,#5f6368)}
.result{fill:var(--dot-result,#6b2fb3)}
.out{fill:var(--dot-result,#6b2fb3);stroke:none}
`;

/** Port offsets along a block's top edge: x, y, z for an FA; x, y for an HA. */
const ports = (kind: 'fa' | 'ha') => (kind === 'fa' ? [1 / 6, 3 / 6, 5 / 6] : [1 / 4, 3 / 4]);

export function diagramSvg(d: Diagram, o: DiagramOptions): string {
	const { g, top, centre, headY, inputsH, gates, outY } = layout(d, o);
	const box = new Map(g.blocks.map((b) => [b.cell, b]));
	const critical = new Set(o.critical ? d.critical : []);
	const out: string[] = [];
	const text = (x: number, y: number, cls: string, s: string, anchor = 'middle') =>
		`<text x="${x}" y="${y}" class="${cls}" text-anchor="${anchor}">${esc(s)}</text>`;

	// Input block (DIA-02).
	out.push(
		`<g class="inputs"><rect class="box" x="${M}" y="${M}" width="${g.width - 2 * M}" height="${inputsH}" rx="4"/>`,
		text(g.width / 2, M + 20, 'caption', o.text.inputs)
	);
	for (const a of gates) {
		const { x, y } = a;
		out.push(
			`<g class="gate"><title>${esc(a.ident)}</title>`,
			`<path class="box" d="M${x} ${y}h${GATE_W - GATE_H / 2}a${GATE_H / 2} ${GATE_H / 2} 0 0 1 0 ${GATE_H}h-${GATE_W - GATE_H / 2}z"/>`,
			text(x + (GATE_W - GATE_H / 2) / 2 + 4, y + GATE_H / 2 + 4, 'wire', a.factors.join('·')),
			'</g>'
		);
	}
	out.push('</g>');

	// Weights 2^j over the columns.
	for (let c = top; c >= 0; c--) {
		out.push(
			`<text x="${centre(c)}" y="${headY}" class="weight" text-anchor="middle">2<tspan dy="-5" font-size="8">${c}</tspan></text>`
		);
	}

	// Where each wire leaves its source: sum from the right half, carry from the left half.
	const source = (from: DiagramEnd, label: string): [number, number] | null => {
		if (typeof from !== 'number') return null;
		const b = box.get(from);
		if (!b) return null;
		const carry = label.startsWith('k');
		return [b.x + (carry ? b.w / 3 : (2 * b.w) / 3), b.y + b.h];
	};
	const used = new Map<number, number>(); // next free input port per block
	const wires: string[] = [];
	const labels: string[] = [];
	for (const e of d.edges) {
		let tx: number;
		let ty: number;
		let stagger = 0;
		if (typeof e.to === 'number') {
			const b = box.get(e.to);
			if (!b) continue;
			const i = used.get(e.to) ?? 0;
			used.set(e.to, i + 1);
			tx = b.x + ports(b.kind)[Math.min(i, ports(b.kind).length - 1)] * b.w;
			ty = b.y;
			stagger = (i % 2) * 10;
		} else if (typeof e.to === 'string' && e.to.startsWith('R')) {
			tx = centre(Number(e.to.slice(1)));
			ty = outY;
		} else continue;
		const from = typeof e.from === 'number' ? box.get(e.from) : undefined;
		const to = typeof e.to === 'number' ? box.get(e.to) : undefined;
		if (from && to && from.y === to.y) {
			// A carry along a row (the CPA): from the side of one block into the next.
			const y = from.y + from.h / 2;
			const [x1, x2] = from.x > to.x ? [from.x, to.x + to.w] : [from.x + from.w, to.x];
			wires.push(`<path class="line" d="M${x1} ${y}H${x2}"/>`);
			labels.push(text((x1 + x2) / 2, y - 4, 'wire', e.label));
			continue;
		}
		const labelY = ty - 6 - stagger;
		const s = source(e.from, e.label);
		if (s) {
			const dy = Math.max(20, (ty - s[1]) / 2);
			wires.push(
				`<path class="line" d="M${s[0]} ${s[1]}C${s[0]} ${s[1] + dy} ${tx} ${ty - dy} ${tx} ${ty}"/>`
			);
		} else {
			// From the input block: a short stub under the label, joined by name.
			wires.push(`<path class="line" d="M${tx} ${labelY + 3}V${ty}"/>`);
		}
		labels.push(text(tx, labelY, 'wire', e.label));
	}
	out.push(...wires);

	for (const b of g.blocks) {
		const name = `${b.kind.toUpperCase()}${b.cell}`;
		out.push(
			`<g class="block ${b.kind}${critical.has(b.cell) ? ' critical' : ''}" data-cell="${b.cell}"><title>${name}</title>`,
			`<rect class="box" x="${b.x}" y="${b.y}" width="${b.w}" height="${b.h}" rx="3"/>`,
			text(b.x + b.w / 2, b.y + b.h / 2 + 4, 'name', name),
			'</g>'
		);
	}
	out.push(...labels);

	// Result bits R(j) once complete (CMP-12).
	if (d.complete) {
		d.outputs.forEach((w, j) => {
			const x = centre(j);
			out.push(
				`<circle class="out" cx="${x}" cy="${outY + 4}" r="4"/>`,
				text(x, outY + 22, 'name result', w === null ? `R(${j}) = 0` : `R(${j})`)
			);
		});
	}

	return (
		`<svg xmlns="http://www.w3.org/2000/svg" width="${g.width}" height="${g.height}" viewBox="0 0 ${g.width} ${g.height}" role="img">` +
		`<title>${esc(o.text.title)}</title><style>${STYLE}</style>` +
		out.join('') +
		'</svg>'
	);
}
