<!-- Caminho relativo: app/web/src/lib/components/Comparison.svelte -->
<!-- MT-38 (minimal): the student's circuit side by side with the automatic solutions already
     revealed. It reveals nothing new (GER-01): a solution enters with step 1 of its help (its
     metrics) and its height table with step 2 (its levels); nothing here is counted. -->
<script lang="ts">
	import type { CompressionSession, HelpLog, Method, Snapshot } from '#lib/core.ts';
	import { STAGE_KEY, methodTexts } from '#lib/methods.ts';
	import { figures, nonDominated, type Figures } from '#lib/comparison.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import HeightTable from './HeightTable.svelte';

	let {
		session,
		snap,
		helps
	}: {
		session: CompressionSession;
		/** The student's latest snapshot. */
		snap: Snapshot;
		/** The help log: a new one means a solution may have been revealed. */
		helps: HelpLog;
	} = $props();

	const id = $props.id();
	// The optimal solution exists only for a height profile (AJU-12).
	const METHODS: Method[] = $derived(
		snap.profile ? ['dadda', 'wallace', 'optimal'] : ['dadda', 'wallace']
	);

	interface Shown {
		method: Method;
		snap: Snapshot;
		heights: ReturnType<CompressionSession['heightTable']> | null;
	}

	/** The solutions already revealed, read again uncounted; they depend only on the matrix. */
	const shown = $derived.by((): Shown[] => {
		void helps;
		const revealed = session.revealed();
		return METHODS.flatMap((method) => {
			const s = session.shownSolution(method);
			if (!s) return [];
			try {
				const path = revealed[STAGE_KEY[method]] === 'path';
				return [{ method, snap: s.snapshot(), heights: path ? s.heightTable() : null }];
			} finally {
				s.free();
			}
		});
	});

	const columns = $derived([
		{
			label: figures(snap).complete ? m.comparison_student() : m.comparison_student_incomplete(),
			f: figures(snap)
		},
		...shown.map((s) => ({
			label: methodTexts(s.method).comparison,
			f: figures(s.snap)
		}))
	]);
	const pareto = $derived(nonDominated(columns.map((c) => c.f)));
	const missing = $derived(METHODS.filter((method) => !shown.some((s) => s.method === method)));

	const number = $derived(
		new Intl.NumberFormat(getLocale() === 'pt-br' ? 'pt-BR' : 'en', { maximumFractionDigits: 2 })
	);
	const ROWS: [() => string, (f: Figures) => number | null][] = [
		[m.comparison_area, (f) => f.area],
		[m.comparison_delay, (f) => f.delay],
		[() => 'FA', (f) => f.fa],
		[() => 'HA', (f) => f.ha],
		[() => 'AND', (f) => f.andGates],
		[m.comparison_levels, (f) => f.levels],
		[m.comparison_cpa_width, (f) => f.cpaWidth],
		[m.comparison_critical, (f) => f.critical],
		[m.comparison_area_delay, (f) => f.areaDelay],
		[m.comparison_area_delay2, (f) => f.areaDelay2]
	];
	const show = (v: number | null) => (v === null ? '—' : number.format(v));
	const yesNo = (v: boolean | null) =>
		v === null ? '—' : v ? m.comparison_yes() : m.comparison_no();

	/** The height table of a solution as Markdown, read again uncounted. */
	function markdown(method: Method) {
		return (headers: Parameters<CompressionSession['heightTableMarkdown']>[0]) => {
			const s = session.shownSolution(method);
			if (!s) return '';
			try {
				return s.heightTableMarkdown(headers);
			} finally {
				s.free();
			}
		};
	}
</script>

<section class="comparison" aria-labelledby={`${id}-title`} data-testid="comparison">
	<h3 id={`${id}-title`}>{m.comparison_title()}</h3>
	{#if shown.length === 0}
		<p class="note" data-testid="comparison-empty">{m.comparison_empty()}</p>
	{:else}
		<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
		<div class="table-wrap" role="region" aria-label={m.comparison_title()} tabindex="0">
			<table data-testid="comparison-table" aria-labelledby={`${id}-title`}>
				<thead>
					<tr>
						<th scope="col">{m.comparison_measure()}</th>
						{#each columns as c, i (i)}
							<th scope="col">{c.label}</th>
						{/each}
					</tr>
				</thead>
				<tbody>
					{#each ROWS as [label, value], r (r)}
						<tr>
							<th scope="row">{label()}</th>
							{#each columns as c, i (i)}
								<td>{show(value(c.f))}</td>
							{/each}
						</tr>
					{/each}
					<tr>
						<th scope="row">{m.comparison_pareto()}</th>
						{#each pareto as p, i (i)}
							<td class:best={p === true}>{yesNo(p)}</td>
						{/each}
					</tr>
				</tbody>
			</table>
		</div>
		<p class="note">{m.comparison_pareto_note()}</p>
		{#if shown.some((s) => s.method === 'dadda')}
			<p class="note">{m.comparison_dadda_note()}</p>
		{/if}
		{#if snap.profile}
			<p class="note">{m.comparison_profile_note()}</p>
		{/if}
		{#each missing as method (method)}
			<p class="note">{methodTexts(method).missing}</p>
		{/each}
		{#each shown as s (s.method)}
			{#if s.heights}
				<HeightTable
					rows={s.heights}
					markdown={markdown(s.method)}
					title={methodTexts(s.method).heightsTitle}
					exportLabel={methodTexts(s.method).heightsExport}
					filename={`dotree-alturas-${s.method}.md`}
					testid={`height-table-${s.method}`}
				/>
			{/if}
		{/each}
	{/if}
</section>

<style>
	.comparison {
		display: grid;
		gap: var(--space-2);
		margin-top: var(--space-3);
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
	td.best {
		font-weight: 600;
	}
</style>
