<!-- Caminho relativo: app/web/src/lib/components/ViewToggle.svelte -->
<!-- Dots or indexed variables (VIS-01). The V shortcut itself lives in the page (VIS-03). -->
<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import type { ViewMode } from '#lib/view.ts';

	let { mode, onchange }: { mode: ViewMode; onchange: (mode: ViewMode) => void } = $props();

	const options: { mode: ViewMode; label: () => string }[] = [
		{ mode: 'dots', label: m.view_dots },
		{ mode: 'indexed', label: m.view_indexed }
	];
</script>

<div class="toggle" role="group" aria-label={m.view_label()}>
	{#each options as o (o.mode)}
		<button
			type="button"
			aria-pressed={mode === o.mode}
			aria-keyshortcuts="V"
			onclick={() => onchange(o.mode)}
		>
			{o.label()}
		</button>
	{/each}
	<span class="hint">{m.view_shortcut()}</span>
</div>

<style>
	.toggle {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-1);
	}
	button {
		font: inherit;
		font-size: 0.9rem;
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius);
		border: 1px solid var(--color-border);
		background: var(--color-surface);
		color: var(--color-text);
		cursor: pointer;
	}
	button[aria-pressed='true'] {
		border-color: var(--color-accent);
		background: var(--color-accent);
		color: var(--color-bg);
	}
	.hint {
		color: var(--color-muted);
		font-size: 0.85rem;
	}
</style>
