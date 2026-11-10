<!-- Caminho relativo: app/web/src/lib/components/HeightTable.svelte -->
<!-- The height table (spec 02, CMP-18): the height of each column at each level, with the
     cells formed over it, the tabular form of the course reference (Parhami, fig. 8.11).
     For the student's own circuit it is always visible and exportable; the comparison
     (MT-38) shows one per solution whose levels were revealed. -->
<script lang="ts">
	import type { HeightHeaders, HeightRow } from '#lib/core.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { download } from '#lib/download.ts';

	let {
		rows,
		markdown,
		title = m.heights_title(),
		exportLabel = m.export_heights(),
		filename = 'dotree-alturas.md',
		testid = 'height-table'
	}: {
		rows: HeightRow[];
		/** The same table as Markdown, from the core, with words in this language. */
		markdown: (headers: HeightHeaders) => string;
		title?: string;
		exportLabel?: string;
		filename?: string;
		testid?: string;
	} = $props();

	const id = $props.id();
	/** Highest column first, as in the level panels. */
	const columns = $derived(rows.length === 0 ? [] : [...rows[0].heights.keys()].reverse());

	function label(r: HeightRow): string {
		switch (r.kind) {
			case 'level':
				return String(r.level);
			case 'cpa':
				return `${r.level} (CPA)`;
			case 'current':
				return `${r.level} (${m.heights_current()})`;
			case 'result':
				return m.heights_result();
		}
	}

	function exportMarkdown() {
		const headers = {
			level: m.table_level(),
			current: m.heights_current(),
			result: m.heights_result()
		};
		download(filename, markdown(headers), 'text/markdown');
	}
</script>

<section class="heights" aria-labelledby={`${id}-title`}>
	<h3 id={`${id}-title`}>{title}</h3>
	<div class="tools">
		<button type="button" onclick={exportMarkdown}>{exportLabel}</button>
	</div>
	<!-- Scrolls sideways on narrow screens, so it takes focus for the arrow keys (GER-07). -->
	<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
	<div class="table-wrap" role="region" aria-label={title} tabindex="0">
		<table data-testid={testid} aria-labelledby={`${id}-title`}>
			<thead>
				<tr>
					<th scope="col">{m.table_level()}</th>
					{#each columns as j (j)}
						<th scope="col">2<sup>{j}</sup></th>
					{/each}
					<th scope="col">FA</th>
					<th scope="col">HA</th>
				</tr>
			</thead>
			<tbody>
				{#each rows as r (r.level)}
					<tr class:current={r.kind === 'current'}>
						<th scope="row">{label(r)}</th>
						{#each columns as j (j)}
							<td>{r.heights[j]}</td>
						{/each}
						<td>{r.kind === 'result' ? '—' : r.fa}</td>
						<td>{r.kind === 'result' ? '—' : r.ha}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
</section>

<style>
	.heights {
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
	.table-wrap {
		overflow-x: auto;
	}
	table {
		border-collapse: collapse;
		font-variant-numeric: tabular-nums;
		font-size: 0.9rem;
	}
	th,
	td {
		padding: 2px var(--space-2);
		border-bottom: 1px solid var(--color-border);
		text-align: right;
		white-space: nowrap;
	}
	th[scope='row'],
	thead th:first-child {
		text-align: left;
	}
	tr.current th,
	tr.current td {
		font-weight: 600;
	}
</style>
