<!-- Caminho relativo: app/web/src/lib/components/LevelPanel.svelte -->
<!-- One level of the compression, drawn like the class figures (docs/DESIGN.md): highest
     weight on the left, 2^j on top, inputs coloured by term and cell outputs in the generated
     colour (COR-01, COR-02), each cell as a box around the points it consumes (CMP-05).
     Columns run from `top` down to 2^0, so W stays hidden until revealed (EXP-09). In the
     indexed view each point becomes its signal, a₁b₀, in the same place and colour (VIS-01).
     In the current level the points are toggle buttons for selection (GER-07). -->
<script lang="ts">
	import type { LevelPoint } from '#lib/core.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { signalParts } from '#lib/signal.ts';
	import { columnLayout, type Group } from '#lib/levels.ts';
	import type { ViewMode } from '#lib/view.ts';

	let {
		title,
		testid,
		points,
		groups,
		top,
		view,
		heights = false,
		selectable = false,
		selected = [],
		ontoggle = () => {}
	}: {
		title: string;
		testid: string;
		points: LevelPoint[];
		groups: (Group & { label: string })[];
		top: number;
		view: ViewMode;
		heights?: boolean;
		selectable?: boolean;
		selected?: number[];
		ontoggle?: (id: number) => void;
	} = $props();

	const columns = $derived(
		Array.from({ length: top + 1 }, (_, i) => {
			const column = top - i;
			return {
				column,
				height: points.filter((p) => p.column === column).length,
				layout: columnLayout(points, groups, column)
			};
		})
	);

	function color(p: LevelPoint): string {
		return p.term === null ? 'var(--dot-generated)' : `var(--dot-term-${(p.term % 8) + 1})`;
	}
</script>

<!-- a₁b₀: letter and subscript per factor; any other shape is written as is. -->
{#snippet indexed(signal: string)}
	{@const parts = signalParts(signal)}
	{#if parts}{#each parts as p, i (i)}{p.name}<sub>{p.index}</sub>{/each}{:else}{signal}{/if}
{/snippet}

{#snippet dot(p: LevelPoint)}
	{#if selectable}
		<button
			type="button"
			class="dot"
			class:indexed={view === 'indexed'}
			style:--c={color(p)}
			aria-pressed={selected.includes(p.id)}
			aria-label={p.signal}
			title={p.signal}
			data-signal={p.signal}
			onclick={() => ontoggle(p.id)}
			>{#if view === 'indexed'}{@render indexed(p.signal)}{/if}</button
		>
	{:else}
		<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
		<span
			class="dot"
			class:indexed={view === 'indexed'}
			style:--c={color(p)}
			role="img"
			tabindex="0"
			aria-label={p.signal}
			title={p.signal}
			data-signal={p.signal}
			>{#if view === 'indexed'}{@render indexed(p.signal)}{/if}</span
		>
	{/if}
{/snippet}

<section class="panel" data-testid={testid} aria-label={title}>
	<h3>{title}</h3>
	<div class="matrix">
		{#each columns as col (col.column)}
			<div
				class="column"
				role="group"
				aria-label={m.matrix_column_label({ column: col.column, height: col.height })}
			>
				<div class="weight">2<sup>{col.column}</sup></div>
				<div class="dots">
					{#each col.layout.groups as g (g.index)}
						<div class="cell {g.kind}" role="group" aria-label={groups[g.index].label}>
							{#each g.points as p (p.id)}{@render dot(p)}{/each}
						</div>
					{/each}
					{#each col.layout.loose as p (p.id)}{@render dot(p)}{/each}
				</div>
				{#if heights}
					<div class="height" aria-hidden="true">{col.height}</div>
				{/if}
			</div>
		{/each}
	</div>
</section>

<style>
	.panel {
		display: grid;
		gap: var(--space-1);
	}
	h3 {
		margin: 0;
		font-size: 1rem;
	}
	.matrix {
		display: flex;
		gap: var(--space-2);
		align-items: flex-start;
		overflow-x: auto;
		padding: var(--space-2) var(--space-1);
	}
	.column {
		display: flex;
		flex-direction: column;
		align-items: center;
		min-width: 2.25rem;
	}
	.weight,
	.height {
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
		color: var(--color-muted);
	}
	.weight {
		margin-bottom: var(--space-1);
	}
	.height {
		margin-top: var(--space-1);
		font-size: 0.85rem;
	}
	.dots,
	.cell {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
	}
	.cell {
		padding: 3px;
		border: 1.5px solid var(--color-text);
		border-radius: 3px;
	}
	.cell.ha {
		border-style: dashed;
	}
	.dot {
		position: relative;
		display: block;
		width: 14px;
		height: 14px;
		padding: 0;
		border: 0;
		background: var(--c);
		font: inherit;
		cursor: default;
	}
	button.dot {
		cursor: pointer;
	}
	.dot[aria-pressed='true'] {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}
	.dot.indexed {
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
	.dot:not(.indexed):focus-visible::after {
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
