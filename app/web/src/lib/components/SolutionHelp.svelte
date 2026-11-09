<!-- Caminho relativo: app/web/src/lib/components/SolutionHelp.svelte -->
<!-- Lower bound and automatic solutions (spec 03): all depend only on the initial matrix and
     come in two steps, result first and path on a second request, each confirmed (GER-03).
     Dadda's solution (AJU-05) and Wallace's (AJU-10) are revealed and used apart, each in a
     MethodSolution. A reopened exercise shows what was already revealed, uncounted (SAL-02). -->
<script lang="ts">
	import { untrack } from 'svelte';
	import type { CompressionSession, LowerBound, Method } from '#lib/core.ts';
	import { m } from '#lib/paraglide/messages.js';
	import type { ViewMode } from '#lib/view.ts';
	import ConfirmDialog from './ConfirmDialog.svelte';
	import MethodSolution from './MethodSolution.svelte';

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
		/** The circuit was replaced by the solution of `method` (AJU-09). */
		onapply: (method: Method) => void;
		/** Below 768 px (GER-10): the circuit cannot be changed from here either. */
		readOnly?: boolean;
	} = $props();

	let boundDialog: ConfirmDialog;
	let boundPathDialog: ConfirmDialog;

	// The parent re-creates this component for each session, so these are read once.
	const shown = untrack(() => session.revealed());
	let bound = $state.raw<LowerBound | null>(untrack(() => session.shownLowerBound()));
	let boundPath = $state(shown.lowerBound === 'path');

	const METHODS: Method[] = ['dadda', 'wallace'];

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
</script>

<div class="solution-help">
	<div class="buttons">
		<button type="button" class="secondary" onclick={askBound}>{m.help_lower_bound()}</button>
		{#if bound && !boundPath}
			<button type="button" class="secondary" onclick={askBoundPath}>
				{m.help_lower_bound_path()}
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

	{#each METHODS as method (method)}
		<MethodSolution {session} {method} {view} {onhelp} onapply={() => onapply(method)} {readOnly} />
	{/each}
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
</style>
