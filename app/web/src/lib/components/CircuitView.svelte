<!-- Caminho relativo: app/web/src/lib/components/CircuitView.svelte -->
<!-- The circuit built so far (spec 05): block diagram and wiring table of the concluded levels,
     with SVG and Markdown exports once complete (DIA-06, DIA-07). The core lays it out on a
     grid; `diagram.ts` draws it. Redrawn whenever the parent passes a new snapshot. -->
<script lang="ts">
	import type { CompressionSession, Snapshot } from '#lib/core.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { diagramSvg } from '#lib/diagram.ts';
	import { download } from '#lib/download.ts';

	let {
		session,
		snap,
		top,
		critical
	}: {
		session: CompressionSession;
		/** The latest snapshot: a new one means the circuit changed. */
		snap: Snapshot;
		/** Highest column drawn, as in the level panels. */
		top: number;
		/** Highlight the critical cells (DIA-05), shared with the level panels. */
		critical: boolean;
	} = $props();

	const id = $props.id();
	// Kept in a constant: Svelte trims literal spaces at the edge of inline markup.
	const ARROW = ' → ';
	let expandAnds = $state(false);

	const diagram = $derived.by(() => {
		void snap;
		return session.diagram();
	});
	const rows = $derived.by(() => {
		void snap;
		return session.wiringTable();
	});
	const svg = $derived(
		diagramSvg(diagram, {
			top,
			critical,
			expandAnds,
			text: {
				title: m.diagram_title(),
				inputs:
					diagram.andGates > 0
						? m.diagram_inputs({ count: diagram.andGates })
						: m.diagram_inputs_only()
			}
		})
	);
	const headers = $derived({
		cell: m.table_cell(),
		level: m.table_level(),
		column: m.table_column(),
		sum: m.table_sum(),
		carry: m.table_carry()
	});

	function exportSvg() {
		download('dotree-diagrama.svg', svg, 'image/svg+xml');
	}

	function exportMarkdown() {
		download('dotree-ligacoes.md', session.wiringTableMarkdown(headers), 'text/markdown');
	}
</script>

{#snippet wire(name: string | null, result: number | null)}
	{#if name === null}—{:else}<code>{name}</code>{#if result !== null}{ARROW}<code>R({result})</code
			>{/if}{/if}
{/snippet}

<section class="circuit" aria-labelledby={`${id}-title`}>
	<h3 id={`${id}-title`}>{m.circuit_title()}</h3>

	{#if diagram.blocks.length === 0}
		<p class="note">{m.diagram_empty()}</p>
	{:else}
		<div class="tools">
			{#if diagram.andGates > 0}
				<button type="button" aria-pressed={expandAnds} onclick={() => (expandAnds = !expandAnds)}>
					{m.action_expand_ands()}
				</button>
			{/if}
			<button
				type="button"
				onclick={exportSvg}
				disabled={!diagram.complete}
				aria-describedby={diagram.complete ? undefined : `${id}-incomplete`}
				>{m.export_svg()}</button
			>
			<button
				type="button"
				onclick={exportMarkdown}
				disabled={!diagram.complete}
				aria-describedby={diagram.complete ? undefined : `${id}-incomplete`}
				>{m.export_markdown()}</button
			>
		</div>
		{#if !diagram.complete}
			<p id={`${id}-incomplete`} class="note" role="status">
				{m.diagram_incomplete({ level: snap.levels.length - 1 })}
			</p>
		{/if}

		<!-- Scrolls sideways on narrow screens, so it takes focus for the arrow keys (GER-07). -->
		<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
		<div
			class="drawing"
			data-testid="diagram"
			role="region"
			aria-label={m.diagram_title()}
			tabindex="0"
		>
			<!-- Generated from the core's layout; every text in it is escaped by diagram.ts. -->
			<!-- eslint-disable-next-line svelte/no-at-html-tags -->
			{@html svg}
		</div>

		<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
		<div class="table-wrap" role="region" aria-label={m.table_title()} tabindex="0">
			<table data-testid="wiring-table">
				<caption>{m.table_title()}</caption>
				<thead>
					<tr>
						<th scope="col">{headers.cell}</th>
						<th scope="col">{headers.level}</th>
						<th scope="col">{headers.column}</th>
						<th scope="col">x</th>
						<th scope="col">y</th>
						<th scope="col">z</th>
						<th scope="col">{headers.sum}</th>
						<th scope="col">{headers.carry}</th>
					</tr>
				</thead>
				<tbody>
					{#each rows as r (r.cell)}
						<tr>
							<td>{r.cell}</td>
							<td>{r.level ?? 'CPA'}</td>
							<td>2<sup>{r.column}</sup></td>
							{#each r.inputs as input, i (i)}
								<td>{@render wire(input, null)}</td>
							{/each}
							<td>{@render wire(r.sum, r.sumResult)}</td>
							<td>{@render wire(r.carry, r.carryResult)}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</section>

<style>
	.circuit {
		display: grid;
		gap: var(--space-2);
		margin-top: var(--space-3);
		padding-top: var(--space-3);
		border-top: 1px solid var(--color-border);
	}
	h3 {
		margin: 0;
		font-size: 1rem;
	}
	.note {
		margin: 0;
		color: var(--color-muted);
		font-size: 0.9rem;
	}
	.tools {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
	button {
		font: inherit;
		padding: var(--space-1) var(--space-3);
		border-radius: var(--radius);
		border: 1px solid var(--color-accent);
		background: transparent;
		color: var(--color-accent);
		cursor: pointer;
	}
	button[aria-pressed='true'] {
		background: var(--color-accent);
		color: var(--color-bg);
	}
	button:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}
	.drawing,
	.table-wrap {
		overflow-x: auto;
	}
	table {
		border-collapse: collapse;
		font-variant-numeric: tabular-nums;
		font-size: 0.9rem;
	}
	caption {
		text-align: left;
		font-weight: 600;
		padding-bottom: var(--space-1);
	}
	th,
	td {
		padding: 2px var(--space-2);
		border-bottom: 1px solid var(--color-border);
		text-align: left;
		white-space: nowrap;
	}
	code {
		font-family: var(--font-mono);
	}
</style>
