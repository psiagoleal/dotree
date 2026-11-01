<!-- Caminho relativo: app/web/src/lib/components/SolutionHelp.svelte -->
<!-- Lower bound and automatic solution (spec 03): both depend only on the initial matrix and
     come in two steps, result first and path on a second request, each confirmed (GER-03).
     The solution is a separate session from the core; the student's circuit is untouched
     (AJU-03). -->
<script lang="ts">
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
		onhelp
	}: { session: CompressionSession; view: ViewMode; onhelp: () => void } = $props();

	let boundDialog: ConfirmDialog;
	let boundPathDialog: ConfirmDialog;
	let solutionDialog: ConfirmDialog;
	let solutionPathDialog: ConfirmDialog;

	let bound = $state.raw<LowerBound | null>(null);
	let boundPath = $state(false);
	let solution = $state.raw<Snapshot | null>(null);
	let solutionPath = $state(false);

	const KIND = { fa: 'FA', ha: 'HA' } as const;
	const halves = (n: number) => formatHalves(n, getLocale());

	/** Reads the solution's snapshot and frees the WASM session right away. */
	function solve(path: boolean): Snapshot {
		const s = session.solution(path);
		try {
			return s.snapshot();
		} finally {
			s.free();
		}
	}

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
