// Caminho relativo: app/web/src/lib/zoom.spec.ts
import { describe, expect, it } from 'vitest';
import { ZOOM_MAX, ZOOM_MIN, fitZoom, zoomIn, zoomOut, zoomText } from './zoom.ts';

describe('zoom steps (VIS-05, DIA-08)', () => {
	it('goes to the next step, from a step or from between steps', () => {
		expect(zoomOut(1)).toBe(0.75);
		expect(zoomIn(1)).toBe(1.5);
		expect(zoomOut(0.6)).toBe(0.5);
		expect(zoomIn(0.6)).toBe(0.75);
	});

	it('stops at the ends', () => {
		expect(zoomOut(ZOOM_MIN)).toBe(ZOOM_MIN);
		expect(zoomIn(ZOOM_MAX)).toBe(ZOOM_MAX);
	});

	it('fits the natural width in the space, never above 100% nor below the minimum', () => {
		expect(fitZoom(928, 2772)).toBeCloseTo(928 / 2772);
		expect(fitZoom(928, 500)).toBe(1);
		expect(fitZoom(100, 36008)).toBe(ZOOM_MIN);
		expect(fitZoom(928, 0)).toBe(1);
		// Nothing drawn and no room (a hidden panel): 0/0 must not become NaN.
		expect(fitZoom(0, 0)).toBe(1);
	});

	it('writes the percentage', () => {
		expect(zoomText(1)).toBe('100%');
		expect(zoomText(928 / 2772)).toBe('33%');
	});
});
