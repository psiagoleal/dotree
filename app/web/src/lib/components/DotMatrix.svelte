<!-- Caminho relativo: app/web/src/lib/components/DotMatrix.svelte -->
<!-- The level-0 dot diagram, drawn like the class figures (docs/DESIGN.md): highest weight on
     the left, 2^j on top, one colour per term (COR-01). Until W is revealed (`width`), only
     columns up to the highest one with dots are drawn (EXP-09). In the indexed view each dot
     becomes its signal, a₁b₀, in the same place and colour (VIS-01, VIS-02). -->
<script lang="ts">
	import type { Dot, Matrix } from '#lib/core.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { signalParts } from '#lib/signal.ts';
	import type { ViewMode } from '#lib/view.ts';

	let {
		matrix,
		view = 'dots',
		width = null
	}: { matrix: Matrix; view?: ViewMode; width?: number | null } = $props();

	const top = $derived(Math.max(0, (width ?? 0) - 1, ...matrix.dots.map((d) => d.column)));
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

<!-- a₁b₀: letter and subscript per factor; any other shape is written as is. -->
{#snippet indexed(signal: string)}
	{@const parts = signalParts(signal)}
	{#if parts}{#each parts as p, i (i)}{p.name}<sub>{p.index}</sub>{/each}{:else}{signal}{/if}
{/snippet}

<div class="matrix" data-testid="dot-matrix">
	{#each columns as col (col.column)}
		<div
			class="column"
			role="group"
			aria-label={m.matrix_column_label({ column: col.column, height: col.dots.length })}
		>
			<div class="weight">2<sup>{col.column}</sup></div>
			<div class="dots" class:indexed={view === 'indexed'}>
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
						>{#if view === 'indexed'}{@render indexed(d.signal)}{/if}</span
					>
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
	.indexed {
		gap: 2px;
	}
	.indexed .dot {
		width: auto;
		height: auto;
		background: none;
		color: var(--c);
		font: 600 0.85rem/1.25 var(--font-mono);
		white-space: nowrap;
	}
	sub {
		font-size: 0.7em;
	}
	/* VIS-04: in the dots view the signal shows on hover (title) and on keyboard focus. */
	.dots:not(.indexed) .dot:focus-visible::after {
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
