// Caminho relativo: app/web/src/lib/zoom.ts
// Zoom of the matrix (VIS-05) and of the block diagram (DIA-08): fixed steps, and "fit to
// width", which may land between two steps. Below 25% neither a dot nor a label can be read.

export const ZOOM_STEPS = [0.25, 0.5, 0.75, 1, 1.5] as const;
export const ZOOM_MIN = ZOOM_STEPS[0];
export const ZOOM_MAX = ZOOM_STEPS[ZOOM_STEPS.length - 1];

/** The next step below `z`, or the minimum. */
export function zoomOut(z: number): number {
	return [...ZOOM_STEPS].reverse().find((s) => s < z - 1e-9) ?? ZOOM_MIN;
}

/** The next step above `z`, or the maximum. */
export function zoomIn(z: number): number {
	return ZOOM_STEPS.find((s) => s > z + 1e-9) ?? ZOOM_MAX;
}

/** The zoom that shows `natural` pixels (at 100%) in `available` ones, from 25% to 100%. */
export function fitZoom(available: number, natural: number): number {
	if (natural <= 0) return 1;
	return Math.min(1, Math.max(ZOOM_MIN, available / natural));
}

export const zoomText = (z: number) => `${Math.round(z * 100)}%`;
