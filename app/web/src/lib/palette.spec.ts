// Caminho relativo: app/web/src/lib/palette.spec.ts
// COR-03 and docs/DESIGN.md: the dot palette, read from app.css, in both themes.
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const css = readFileSync(new URL('../app.css', import.meta.url), 'utf8');

/** Custom properties declared in a block of the stylesheet. */
function tokens(block: string): Record<string, string> {
	return Object.fromEntries(
		[...block.matchAll(/--([\w-]+):\s*(#[0-9a-fA-F]{6})/g)].map((m) => [m[1], m[2].toLowerCase()])
	);
}

const darkStart = css.indexOf('@media (prefers-color-scheme: dark)');
const themes = {
	light: tokens(css.slice(0, darkStart)),
	dark: tokens(css.slice(darkStart))
};

function channel(c: number): number {
	const v = c / 255;
	return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
}
function rgb(hex: string): [number, number, number] {
	return [1, 3, 5].map((i) => channel(parseInt(hex.slice(i, i + 2), 16))) as [
		number,
		number,
		number
	];
}
function luminance(hex: string): number {
	const [r, g, b] = rgb(hex);
	return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}
/** WCAG contrast ratio. */
function contrast(a: string, b: string): number {
	const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
	return (hi + 0.05) / (lo + 0.05);
}
/** CIE76 colour difference in CIELAB (D65). */
function deltaE(a: string, b: string): number {
	const lab = (hex: string) => {
		const [r, g, b2] = rgb(hex);
		const x = (0.4124 * r + 0.3576 * g + 0.1805 * b2) / 0.95047;
		const y = 0.2126 * r + 0.7152 * g + 0.0722 * b2;
		const z = (0.0193 * r + 0.1192 * g + 0.9505 * b2) / 1.08883;
		const f = (t: number) => (t > 0.008856 ? Math.cbrt(t) : 7.787 * t + 16 / 116);
		return [116 * f(y) - 16, 500 * (f(x) - f(y)), 200 * (f(y) - f(z))];
	};
	const [p, q] = [lab(a), lab(b)];
	return Math.hypot(p[0] - q[0], p[1] - q[1], p[2] - q[2]);
}

describe.each(Object.entries(themes))('dot palette, %s theme (COR-03)', (_, t) => {
	const terms = Array.from({ length: 8 }, (_, i) => t[`dot-term-${i + 1}`]);

	it('declares 8 term colors, the generated color and the result color', () => {
		expect(terms.every(Boolean)).toBe(true);
		expect(t['dot-generated']).toBeTruthy();
		expect(t['dot-result']).toBeTruthy();
	});

	it('every dot color has at least 3:1 contrast with the background', () => {
		for (const c of [...terms, t['dot-generated'], t['dot-result']]) {
			expect(contrast(c, t['color-bg']), c).toBeGreaterThanOrEqual(3);
		}
	});

	it('term colors stay away from the generated and result colors', () => {
		for (const c of terms) {
			expect(deltaE(c, t['dot-generated']), c).toBeGreaterThanOrEqual(20);
			expect(deltaE(c, t['dot-result']), c).toBeGreaterThanOrEqual(20);
		}
	});

	it('term colors are distinguishable from each other', () => {
		for (let i = 0; i < terms.length; i++) {
			for (let j = i + 1; j < terms.length; j++) {
				expect(deltaE(terms[i], terms[j]), `${terms[i]} x ${terms[j]}`).toBeGreaterThanOrEqual(25);
			}
		}
	});
});
