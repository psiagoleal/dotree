<!-- Caminho relativo: app/web/src/lib/components/Zoom.svelte -->
<!-- Zoom out, the current zoom, zoom in and "fit to width" (VIS-05, DIA-08). The owner keeps
     the zoom and says how wide its content is at 100% and how much room there is. -->
<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import { ZOOM_MAX, ZOOM_MIN, fitZoom, zoomIn, zoomOut, zoomText } from '#lib/zoom.ts';

	let {
		zoom,
		onchange,
		label,
		measure
	}: {
		zoom: number;
		onchange: (zoom: number) => void;
		/** Names the group: what this control zooms. */
		label: string;
		/** Room available and width of the content at 100%, in pixels; null before drawing. */
		measure: () => { available: number; natural: number } | null;
	} = $props();

	function fit() {
		const size = measure();
		if (size) onchange(fitZoom(size.available, size.natural));
	}
</script>

<div class="zoom" role="group" aria-label={label}>
	<button
		type="button"
		aria-label={m.zoom_out()}
		title={m.zoom_out()}
		disabled={zoom <= ZOOM_MIN}
		onclick={() => onchange(zoomOut(zoom))}>−</button
	>
	<output>{zoomText(zoom)}</output>
	<button
		type="button"
		aria-label={m.zoom_in()}
		title={m.zoom_in()}
		disabled={zoom >= ZOOM_MAX}
		onclick={() => onchange(zoomIn(zoom))}>+</button
	>
	<button type="button" onclick={fit}>{m.zoom_fit()}</button>
</div>

<style>
	.zoom {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-1);
	}
	button {
		font: inherit;
		font-size: 0.9rem;
		min-width: 2rem;
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius);
		border: 1px solid var(--color-border);
		background: var(--color-surface);
		color: var(--color-text);
		cursor: pointer;
	}
	button:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}
	output {
		min-width: 3.5rem;
		text-align: center;
		font-variant-numeric: tabular-nums;
		font-size: 0.9rem;
	}
</style>
