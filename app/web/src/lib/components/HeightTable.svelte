<!-- Caminho relativo: app/web/src/lib/components/HeightTable.svelte -->
<!-- The height table (spec 02, CMP-16): the height of each column at each level, with the
     cells formed over it, the tabular form of the course reference (Parhami, fig. 8.11).
     It shows only the student's own circuit, so it is always visible and exportable.
     Redrawn whenever the parent passes a new snapshot. -->
<script lang="ts">
	import type { CompressionSession, HeightRow, Snapshot } from '#lib/core.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { download } from '#lib/download.ts';

	let {
		session,
		snap
	}: {
		session: CompressionSession;
		/** The latest snapshot: a new one means the circuit changed. */
		snap: Snapshot;
	} = $props();

	const id = $props.id();

	const rows = $derived.by(() => {
		void snap;
		return session.heightTable();
	});
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
		download('dotree-alturas.md', session.heightTableMarkdown(headers), 'text/markdown');
	}
</script>

<section class="heights" aria-labelledby={`${id}-title`}>
	<h3 id={`${id}-title`}>{m.heights_title()}</h3>
	<div class="tools">
		<button type="button" onclick={exportMarkdown}>{m.export_heights()}</button>
	</div>
	<!-- Scrolls sideways on narrow screens, so it takes focus for the arrow keys (GER-07). -->
	<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
	<div class="table-wrap" role="region" aria-label={m.heights_title()} tabindex="0">
		<table data-testid="height-table" aria-labelledby={`${id}-title`}>
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
