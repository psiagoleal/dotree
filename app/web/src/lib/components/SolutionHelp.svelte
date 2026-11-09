<!-- Caminho relativo: app/web/src/lib/components/SolutionHelp.svelte -->
<!-- Lower bound and automatic solution (spec 03): both depend only on the initial matrix and
     come in two steps, result first and path on a second request, each confirmed (GER-03).
     The solution is a separate session from the core; the student's circuit is untouched
     (AJU-03) until the student, having seen the levels, asks to use it (AJU-09). A reopened
     exercise shows what was already revealed, uncounted (SAL-02). -->
<script lang="ts">
	import { untrack } from 'svelte';
	import type { CompressionSession, LowerBound, Snapshot } from '#lib/core.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import { formatHalves } from '#lib/levels.ts';
	import type { ViewMode } from '#lib/view.ts';
	import ConfirmDialog from './ConfirmDialog.svelte';
	import LevelPanel from './LevelPanel.svelte';

	let {
		session,
		view,
		onhelp,
		onapply,
		readOnly = false
	}: {
		session: CompressionSession;
		view: ViewMode;
		onhelp: () => void;
		/** The circuit was replaced by the solution (AJU-09). */
		onapply: () => void;
		/** Below 768 px (GER-10): the circuit cannot be changed from here either. */
		readOnly?: boolean;
	} = $props();

	let boundDialog: ConfirmDialog;
	let boundPathDialog: ConfirmDialog;
	let solutionDialog: ConfirmDialog;
	let solutionPathDialog: ConfirmDialog;
	let applyDialog: ConfirmDialog;

	// The parent re-creates this component for each session, so these are read once.
	const shown = untrack(() => session.revealed());
	let bound = $state.raw<LowerBound | null>(untrack(() => session.shownLowerBound()));
	let boundPath = $state(shown.lowerBound === 'path');
	let solution = $state.raw<Snapshot | null>(
		untrack(() => {
			const s = session.shownSolution();
			return s && read(s);
		})
	);
	let solutionPath = $state(shown.solution === 'path');

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

	const solve = (path: boolean) => read(session.solution(path));

	async function askBound() {
		if (!(await boundDialog.ask())) return;
		bound = session.lowerBound(false);
		onhelp();
	}

	async function askBoundPath() {
		if (!(await boundPathDialog.ask())) return;
		bound = session.lowerBound(true);
		boundPath = true;
		onhelp();
	}

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
		if (session.applySolution()) onapply();
	}

	function summary(s: Snapshot): string {
		const cpa = s.levels.some((l) => l.cpa);
		const args = {
			levels: s.levels.filter((l) => !l.cpa).length,
			fa: s.metrics.fa,
			ha: s.metrics.ha,
			area: halves(s.metrics.area),
			delay: halves(s.metrics.delay)
		};
		return cpa ? m.solution_summary_cpa(args) : m.solution_summary(args);
	}
</script>

<div class="solution-help">
	<div class="buttons">
		<button type="button" class="secondary" onclick={askBound}>{m.help_lower_bound()}</button>
		{#if bound && !boundPath}
			<button type="button" class="secondary" onclick={askBoundPath}>
				{m.help_lower_bound_path()}
			</button>
		{/if}
		<button type="button" class="secondary" onclick={askSolution}>{m.help_solution()}</button>
		{#if solution && !solutionPath}
			<button type="button" class="secondary" onclick={askSolutionPath}>
				{m.help_solution_path()}
			</button>
		{/if}
		{#if solutionPath && !readOnly}
			<button type="button" class="secondary" onclick={apply}>{m.help_use_solution()}</button>
		{/if}
	</div>

	{#if bound}
		<p data-testid="lower-bound">{m.lower_bound_result({ value: bound.value })}</p>
		{#if boundPath}
			<p data-testid="lower-bound-path">
				{m.lower_bound_path({ dots: bound.dots, width: bound.outputWidth, value: bound.value })}
			</p>
		{/if}
	{/if}

	{#if solution}
		<p data-testid="solution-summary">{summary(solution)}</p>
		{#if solutionPath && solution.result}
			{@const top = solution.result.length - 1}
			<section class="solution" data-testid="solution" aria-label={m.solution_title()}>
				<h3>{m.solution_title()}</h3>
				{#each solution.levels as level, i (i)}
					<LevelPanel
						title={level.cpa ? m.solution_cpa({ level: i }) : m.solution_level({ level: i })}
						testid={`solution-level-${i}`}
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
					title={m.solution_result()}
					testid="solution-result"
					points={solution.current.points}
					groups={[]}
					{top}
					{view}
					result={solution.result}
				/>
			</section>
		{/if}
	{/if}
</div>

<ConfirmDialog
	bind:this={boundDialog}
	title={m.confirm_lower_bound_title()}
	body={m.confirm_lower_bound_body()}
/>
<ConfirmDialog
	bind:this={boundPathDialog}
	title={m.confirm_lower_bound_path_title()}
	body={m.confirm_lower_bound_path_body()}
/>
<ConfirmDialog
	bind:this={solutionDialog}
	title={m.confirm_solution_title()}
	body={m.confirm_solution_body()}
/>
<ConfirmDialog
	bind:this={solutionPathDialog}
	title={m.confirm_solution_path_title()}
	body={m.confirm_solution_path_body()}
/>
<ConfirmDialog
	bind:this={applyDialog}
	title={m.confirm_use_solution_title()}
	body={m.confirm_use_solution_body()}
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
	.solution-help {
		display: grid;
		gap: var(--space-2);
	}
	.buttons {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
	p {
		margin: 0;
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
