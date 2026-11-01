<!-- Caminho relativo: app/web/src/lib/components/DotMatrix.svelte -->
<!-- The level-0 dot diagram, drawn like the class figures (docs/DESIGN.md): highest weight on
     the left, 2^j on top, one colour per term (COR-01). Only columns up to the highest one
     with dots are drawn, so the output width W stays hidden (EXP-09). -->
<script lang="ts">
	import type { Dot, Matrix } from '#lib/core.ts';
	import { m } from '#lib/paraglide/messages.js';

	let { matrix }: { matrix: Matrix } = $props();

	const top = $derived(Math.max(0, ...matrix.dots.map((d) => d.column)));
	const columns = $derived(
		Array.from({ length: top + 1 }, (_, i) => {
			const column = top - i;
			return { column, dots: matrix.dots.filter((d) => d.column === column) };
		})
	);

	function color(d: Dot): string {
		return `var(--dot-term-${(d.term % 8) + 1})`;
	}
</script>

<div class="matrix" data-testid="dot-matrix">
	{#each columns as col (col.column)}
		<div
			class="column"
			role="group"
			aria-label={m.matrix_column_label({ column: col.column, height: col.dots.length })}
		>
			<div class="weight">2<sup>{col.column}</sup></div>
			<div class="dots">
				{#each col.dots as d (d.id)}
					<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
					<span
						class="dot"
						style:--c={color(d)}
						role="img"
						tabindex="0"
						aria-label={d.signal}
						title={d.signal}
						data-signal={d.signal}
						data-term={d.term}
					></span>
				{/each}
			</div>
		</div>
	{/each}
</div>

<style>
	.matrix {
		display: flex;
		gap: var(--space-2);
		align-items: flex-start;
		overflow-x: auto;
		padding: var(--space-2) 0;
	}
	.column {
		display: flex;
		flex-direction: column;
		align-items: center;
		min-width: 2.25rem;
	}
	.weight {
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
		color: var(--color-muted);
		margin-bottom: var(--space-1);
	}
	.dots {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.dot {
		position: relative;
		display: block;
		width: 14px;
		height: 14px;
		background: var(--c);
	}
	/* VIS-04: the signal shows on hover (title) and on keyboard focus. */
	.dot:focus-visible::after {
		content: attr(data-signal);
		position: absolute;
		left: 50%;
		top: -1.6rem;
		transform: translateX(-50%);
		padding: 0 var(--space-1);
		font: 0.8rem var(--font-mono);
		color: var(--color-text);
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--radius);
		white-space: nowrap;
		z-index: 1;
	}
</style>
