<!-- Caminho relativo: app/web/src/lib/components/MethodSolution.svelte -->
<!-- The automatic solution of one method (spec 03): Dadda's (AJU-05), Wallace's (AJU-10) or
     the optimal circuit of a height profile (AJU-12).
     Two steps, each confirmed (GER-03): the final metrics, then the levels with the method's
     rule. The solution is a separate session from the core; the student's circuit is
     untouched (AJU-03) until the student, having seen the levels, asks to use it (AJU-09).
     A reopened exercise shows what was already revealed, uncounted (SAL-02). -->
<script lang="ts">
	import { untrack } from 'svelte';
	import type { CompressionSession, Method, Snapshot } from '#lib/core.ts';
	import { STAGE_KEY, methodTexts } from '#lib/methods.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import { formatHalves } from '#lib/levels.ts';
	import type { ViewMode } from '#lib/view.ts';
	import ConfirmDialog from './ConfirmDialog.svelte';
	import LevelPanel from './LevelPanel.svelte';

	let {
		session,
		method,
		view,
		onhelp,
		onapply,
		readOnly = false
	}: {
		session: CompressionSession;
		method: Method;
		view: ViewMode;
		onhelp: () => void;
		/** The circuit was replaced by this solution (AJU-09). */
		onapply: () => void;
		/** Below 768 px (GER-10): the circuit cannot be changed from here either. */
		readOnly?: boolean;
	} = $props();

	let solutionDialog: ConfirmDialog;
	let solutionPathDialog: ConfirmDialog;
	let applyDialog: ConfirmDialog;

	// The parent re-creates this component for each session, so these are read once.
	const t = $derived(methodTexts(method));
	// Dadda keeps the test ids it had before Wallace came (MT-20).
	const tid = untrack(() => (method === 'dadda' ? 'solution' : method));
	const stage = untrack(() => session.revealed()[STAGE_KEY[method]]);
	// The optimum may not exist or be too large to search; asking reveals nothing (AJU-12).
	const search = untrack(() => (method === 'optimal' ? session.optimalSearch() : 'ok'));
	let solution = $state.raw<Snapshot | null>(
		untrack(() => {
			const s = session.shownSolution(method);
			return s && read(s);
		})
	);
	let solutionPath = $state(stage === 'path');

	const KIND = { fa: 'FA', ha: 'HA' } as const;
	const halves = (n: number) => formatHalves(n, getLocale());

	/** Reads the solution's snapshot and frees the WASM session right away. */
	function read(s: CompressionSession): Snapshot {
		try {
			return s.snapshot();
		} finally {
			s.free();
		}
	}

	const solve = (path: boolean) => read(session.solution(path, method));

	async function askSolution() {
		if (!(await solutionDialog.ask())) return;
		solution = solve(false);
		onhelp();
	}

	async function askSolutionPath() {
		if (!(await solutionPathDialog.ask())) return;
		solution = solve(true);
		solutionPath = true;
		onhelp();
	}

	async function apply() {
		if (!(await applyDialog.ask())) return;
		if (session.applySolution(method)) onapply();
	}

	function summary(s: Snapshot): string {
		const args = {
			levels: s.levels.filter((l) => !l.cpa).length,
			fa: s.metrics.fa,
			ha: s.metrics.ha,
			area: halves(s.metrics.area),
			delay: halves(s.metrics.delay)
		};
		return t.summary(
			args,
			s.levels.some((l) => l.cpa)
		);
	}
</script>

{#if search === 'ok'}
	<div class="buttons">
		<button type="button" class="secondary" onclick={askSolution}>{t.ask}</button>
		{#if solution && !solutionPath}
			<button type="button" class="secondary" onclick={askSolutionPath}>{t.askPath}</button>
		{/if}
		{#if solutionPath && !readOnly}
			<button type="button" class="secondary" onclick={apply}>{t.use}</button>
		{/if}
	</div>
{:else if search}
	<p class="note" data-testid={`${tid}-unavailable`}>
		{search === 'tooLarge' ? m.optimal_too_large() : m.optimal_no_circuit()}
	</p>
{/if}

{#if solution}
	<p data-testid={`${tid}-summary`}>{summary(solution)}</p>
	<!-- The optimum stops at the profile, so its output may be redundant, not a result. -->
	{#if solutionPath && (solution.result || solution.profile?.reached)}
		{@const top = solution.heights.length - 1}
		<section class="solution" data-testid={tid} aria-label={t.title}>
			<h3>{t.title}</h3>
			<p data-testid={`${tid}-rule`}>{t.rule}</p>
			{#each solution.levels as level, i (i)}
				<LevelPanel
					title={level.cpa ? t.cpa(i) : t.level(i)}
					testid={`${tid}-level-${i}`}
					points={level.points}
					groups={level.cells.map((c) => ({
						...c,
						label: m.cell_concluded({ kind: KIND[c.kind], number: c.number })
					}))}
					{top}
					{view}
				/>
			{/each}
			<LevelPanel
				title={t.last}
				testid={`${tid}-result`}
				points={solution.current.points}
				groups={[]}
				{top}
				{view}
				result={solution.result}
			/>
		</section>
	{/if}
{/if}

<ConfirmDialog bind:this={solutionDialog} title={t.confirmTitle} body={m.confirm_solution_body()} />
<ConfirmDialog
	bind:this={solutionPathDialog}
	title={t.confirmPathTitle}
	body={m.confirm_solution_path_body()}
/>
<ConfirmDialog
	bind:this={applyDialog}
	title={t.confirmUseTitle}
	body={t.confirmUseBody}
	confirmLabel={m.confirm_use_solution_ok()}
/>

<style>
	button {
		font: inherit;
		padding: var(--space-1) var(--space-3);
		border-radius: var(--radius);
		border: 1px solid var(--color-accent);
		background: transparent;
		color: var(--color-accent);
		cursor: pointer;
	}
	.buttons {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
	p {
		margin: 0;
	}
	.note {
		color: var(--color-muted);
	}
	.solution {
		display: grid;
		gap: var(--space-2);
		padding: var(--space-3);
		border: 1px dashed var(--color-border);
		border-radius: var(--radius);
	}
	h3 {
		margin: 0;
		font-size: 1rem;
	}
</style>
