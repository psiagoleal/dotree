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
		snap.profile ? ['dadda', 'parhami', 'wallace', 'optimal'] : ['dadda', 'parhami', 'wallace']
	);

	interface Shown {
		key: string;
		label: string;
		snap: Snapshot;
		heights: ReturnType<CompressionSession['heightTable']> | null;
		/** The solution again, uncounted, for its height table's Markdown. */
		again: () => CompressionSession | null;
		heightsTitle: string;
		heightsExport: string;
	}

	/** Reads what `open` gives and frees it; nothing when it gives nothing. */
	function reading<T>(open: () => CompressionSession | null, use: (s: CompressionSession) => T) {
		const s = open();
		if (!s) return null;
		try {
			return use(s);
		} finally {
			s.free();
		}
	}

	/** The solutions already revealed, read again uncounted. They depend on the matrix and
	 *  the costs (GER-12); METHODS reads the snapshot, so they are read again with each. */
	const shown = $derived.by((): Shown[] => {
		void helps;
		const revealed = session.revealed();
		const methods = METHODS.flatMap((method): Shown[] => {
			const again = () => session.shownSolution(method);
			const path = revealed[STAGE_KEY[method]] === 'path';
			const t = methodTexts(method);
			const one = reading(again, (s) => ({
				key: method,
				label: t.comparison,
				snap: s.snapshot(),
				heights: path ? s.heightTable() : null,
				again,
				heightsTitle: t.heightsTitle,
				heightsExport: t.heightsExport
			}));
			return one ? [one] : [];
		});
		// The optimizer's options (AJU-14): with step 1, and their height tables with step 2.
		const optimizer = session.shownOptimizer();
		const path = revealed.optimizer === 'path';
		const options = Array.from({ length: optimizer?.count ?? 0 }, (_, i): Shown[] => {
			const n = i + 1;
			const again = () => session.optimizerOption(i);
			const one = reading(again, (s) => ({
				key: `optimizer-${n}`,
				label: m.comparison_optimizer({ n }),
				snap: s.snapshot(),
				heights: path ? s.heightTable() : null,
				again,
				heightsTitle: m.comparison_heights_title_optimizer({ n }),
				heightsExport: m.comparison_heights_export_optimizer({ n })
			}));
			return one ? [one] : [];
		}).flat();
		return [...methods, ...options];
	});

	const columns = $derived([
		{
			label: figures(snap).complete ? m.comparison_student() : m.comparison_student_incomplete(),
			f: figures(snap)
		},
		...shown.map((s) => ({
			label: s.label,
			f: figures(s.snap)
		}))
	]);
	const pareto = $derived(nonDominated(columns.map((c) => c.f)));
	const missing = $derived(METHODS.filter((method) => !shown.some((s) => s.key === method)));
	const optimizerMissing = $derived(!shown.some((s) => s.key.startsWith('optimizer-')));

	const number = $derived(
		new Intl.NumberFormat(getLocale() === 'pt-br' ? 'pt-BR' : 'en', { maximumFractionDigits: 2 })
	);
	// The counter rows appear only when a column used counters (CMP-20).
	const counters = $derived(columns.some((c) => c.f.c73 + c.f.c154 > 0));
	const ROWS = $derived<[() => string, (f: Figures) => number | null][]>([
		[m.comparison_area, (f) => f.area],
		[m.comparison_delay, (f) => f.delay],
		[() => 'FA', (f) => f.fa],
		[() => 'HA', (f) => f.ha],
		...(counters
			? ([
					[() => '7:3', (f) => f.c73],
					[() => '15:4', (f) => f.c154]
				] as [() => string, (f: Figures) => number | null][])
			: []),
		[() => 'AND', (f) => f.andGates],
		[m.comparison_levels, (f) => f.levels],
		[m.comparison_cpa_width, (f) => f.cpaWidth],
		[m.comparison_critical, (f) => f.critical],
		[m.comparison_area_delay, (f) => f.areaDelay],
		[m.comparison_area_delay2, (f) => f.areaDelay2]
	]);
	const show = (v: number | null) => (v === null ? '—' : number.format(v));
	const yesNo = (v: boolean | null) =>
		v === null ? '—' : v ? m.comparison_yes() : m.comparison_no();

	/** The height table of a solution as Markdown, read again uncounted. */
	function markdown(again: () => CompressionSession | null) {
		return (headers: Parameters<CompressionSession['heightTableMarkdown']>[0]) =>
			reading(again, (s) => s.heightTableMarkdown(headers)) ?? '';
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
		{#if shown.some((s) => s.key === 'dadda')}
			<p class="note">{m.comparison_dadda_note()}</p>
		{/if}
		{#if snap.profile}
			<p class="note">{m.comparison_profile_note()}</p>
		{/if}
		{#each missing as method (method)}
			<p class="note">{methodTexts(method).missing}</p>
		{/each}
		{#if optimizerMissing}
			<p class="note">{m.comparison_missing_optimizer()}</p>
		{/if}
		{#each shown as s (s.key)}
			{#if s.heights}
				<HeightTable
					rows={s.heights}
					markdown={markdown(s.again)}
					title={s.heightsTitle}
					exportLabel={s.heightsExport}
					filename={`dotree-alturas-${s.key}.md`}
					testid={`height-table-${s.key}`}
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
