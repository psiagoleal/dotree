<!-- Caminho relativo: app/web/src/lib/components/TermLegend.svelte -->
<!-- One entry per combined term, in its dot colour (COR-01). From the 9th term on the colours
     repeat, so entries are numbered (COR-03). -->
<script lang="ts">
	import type { Term } from '#lib/core.ts';
	import { m } from '#lib/paraglide/messages.js';

	let { terms }: { terms: Term[] } = $props();

	const text = (t: Term) => (t.coefficient === '1' ? '' : t.coefficient) + t.vars;
</script>

<section class="legend">
	<h3>{m.legend_title()}</h3>
	<ol data-testid="term-legend">
		{#each terms as t, i (i)}
			<li>
				<span class="swatch" style:--c={`var(--dot-term-${(i % 8) + 1})`}></span>
				<span class="index">{i + 1}.</span>
				<span class="term">{text(t)}</span>
			</li>
		{/each}
	</ol>
</section>

<style>
	h3 {
		font-size: 1rem;
		margin: 0 0 var(--space-1);
	}
	ol {
		list-style: none;
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1) var(--space-3);
		padding: 0;
		margin: 0;
	}
	li {
		display: flex;
		align-items: center;
		gap: var(--space-1);
	}
	.swatch {
		width: 14px;
		height: 14px;
		background: var(--c);
	}
	.index {
		color: var(--color-muted);
		font-variant-numeric: tabular-nums;
	}
	.term {
		font-family: var(--font-mono);
	}
</style>
