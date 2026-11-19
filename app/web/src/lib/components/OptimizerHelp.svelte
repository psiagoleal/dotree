<!-- Caminho relativo: app/web/src/lib/components/OptimizerHelp.svelte -->
<!-- The optimizer (AJU-14, MT-46): the circuits not dominated in area and delay, under the
     costs in force, searched in the worker when the exercise opens and after new costs.
     Two steps, each confirmed and counted (GER-03): the metrics of every option, which also
     enter the comparison, then their levels, each option with its own "use" (AJU-09).
     The student's circuit is untouched until then. A reopened exercise shows what was
     already revealed, uncounted (SAL-02). -->
<script lang="ts">
	import { untrack } from 'svelte';
	import type { CompressionSession, Optimizer, Snapshot } from '#lib/core.ts';
	import { cellName } from '#lib/cells.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import { formatHundredths } from '#lib/levels.ts';
	import type { ViewMode } from '#lib/view.ts';
	import ConfirmDialog from './ConfirmDialog.svelte';
	import LevelPanel from './LevelPanel.svelte';

	let {
		session,
		view,
		search,
		onhelp,
		onapply,
		readOnly = false
	}: {
		session: CompressionSession;
		view: ViewMode;
		/** Whether the search ended under the current costs. */
		search: 'pending' | 'ok';
		onhelp: () => void;
		/** The circuit was replaced by option `n` (1-based). */
		onapply: (n: number) => void;
		/** Below 768 px (GER-10): the circuit cannot be changed from here either. */
		readOnly?: boolean;
	} = $props();

	let askDialog: ConfirmDialog;
	let pathDialog: ConfirmDialog;
	let useDialog: ConfirmDialog;
	let useN = $state(1);

	// The parent re-creates this component for each session and set of costs; the end of a
	// search comes as `search`, and what a reopened exercise had revealed is read then.
	let shown = $state.raw<Optimizer | null>(untrack(() => session.shownOptimizer()));
	let path = $state(untrack(() => session.revealed().optimizer === 'path'));
	$effect(() => {
		if (search === 'ok') shown = untrack(() => session.shownOptimizer());
	});

	/** Reads an option's snapshot and frees its WASM session right away. */
	function read(s: CompressionSession): Snapshot {
		try {
			return s.snapshot();
		} finally {
			s.free();
		}
	}

	const options = $derived.by((): Snapshot[] => {
		if (!shown) return [];
		return Array.from({ length: shown.count }, (_, i) => read(session.optimizerOption(i)!));
	});

	const hundredths = (n: number) => formatHundredths(n, getLocale());

	function mode(o: Optimizer): string {
		switch (o.mode) {
			case 'exact':
				return m.optimizer_mode_exact();
			case 'exactUpTo':
				return m.optimizer_mode_exact_up_to({ area: hundredths(o.upTo) });
			case 'heuristic':
				return m.optimizer_mode_heuristic();
		}
	}

	function summary(s: Snapshot, n: number): string {
		const k = s.metrics;
		const cells = [
			`${k.fa} FA`,
			`${k.ha} HA`,
			...(k.c73 > 0 ? [`${k.c73} 7:3`] : []),
			...(k.c154 > 0 ? [`${k.c154} 15:4`] : [])
		].join(', ');
		const args = {
			n,
			levels: s.levels.filter((l) => !l.cpa).length,
			cells,
			area: hundredths(k.area),
			delay: hundredths(k.delay)
		};
		return s.levels.some((l) => l.cpa) ? m.optimizer_option_cpa(args) : m.optimizer_option(args);
	}

	async function ask() {
		if (!(await askDialog.ask())) return;
		shown = session.optimizer(false);
		onhelp();
	}

	async function askPath() {
		if (!(await pathDialog.ask())) return;
		if (session.optimizer(true)) path = true;
		onhelp();
	}

	async function use(n: number) {
		useN = n;
		if (!(await useDialog.ask())) return;
		if (session.applyOptimizerOption(n - 1)) onapply(n);
	}
</script>

{#if search === 'pending'}
	<p class="note searching" role="status" data-testid="optimizer-searching">
		<progress aria-label={m.optimizer_searching()}></progress>
		{m.optimizer_searching()}
	</p>
{:else}
	<div class="buttons">
		<button type="button" class="secondary" onclick={ask}>{m.optimizer_ask()}</button>
		{#if shown && !path}
			<button type="button" class="secondary" onclick={askPath}>{m.optimizer_ask_path()}</button>
		{/if}
	</div>
{/if}

{#if shown}
	<p class="note" data-testid="optimizer-mode">{mode(shown)}</p>
	<ol class="options" data-testid="optimizer-options">
		{#each options as o, i (i)}
			<li data-testid={`optimizer-option-${i + 1}`}>{summary(o, i + 1)}</li>
		{/each}
	</ol>
	{#if path}
		<section class="solution" data-testid="optimizer" aria-label={m.optimizer_title()}>
			<h3>{m.optimizer_title()}</h3>
			<p data-testid="optimizer-rule">{m.optimizer_rule()}</p>
			{#each options as o, i (i)}
				{@const n = i + 1}
				{@const top = o.heights.length - 1}
				<section
					class="option"
					data-testid={`optimizer-${n}`}
					aria-label={m.optimizer_option_title({ n })}
				>
					<h4>{m.optimizer_option_title({ n })}</h4>
					{#each o.levels as level, l (l)}
						<LevelPanel
							title={level.cpa
								? m.optimizer_cpa({ n, level: l })
								: m.optimizer_level({ n, level: l })}
							testid={`optimizer-${n}-level-${l}`}
							points={level.points}
							groups={level.cells.map((c) => ({ ...c, label: cellName(c) }))}
							{top}
							{view}
						/>
					{/each}
					<LevelPanel
						title={o.result ? m.optimizer_result({ n }) : m.optimizer_output({ n })}
						testid={`optimizer-${n}-result`}
						points={o.current.points}
						groups={[]}
						{top}
						{view}
						result={o.result}
					/>
					{#if !readOnly}
						<div class="buttons">
							<button type="button" class="secondary" onclick={() => use(n)}>
								{m.optimizer_use({ n })}
							</button>
						</div>
					{/if}
				</section>
			{/each}
		</section>
	{/if}
{/if}

<ConfirmDialog
	bind:this={askDialog}
	title={m.optimizer_confirm_title()}
	body={m.optimizer_confirm_body()}
/>
<ConfirmDialog
	bind:this={pathDialog}
	title={m.optimizer_confirm_path_title()}
	body={m.confirm_solution_path_body()}
/>
<ConfirmDialog
	bind:this={useDialog}
	title={m.optimizer_confirm_use_title({ n: useN })}
	body={m.optimizer_confirm_use_body()}
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
	.options {
		margin: 0;
		padding-left: var(--space-4);
	}
	.solution,
	.option {
		display: grid;
		gap: var(--space-2);
	}
	.solution {
		padding: var(--space-3);
		border: 1px dashed var(--color-border);
		border-radius: var(--radius);
	}
	h3,
	h4 {
		margin: 0;
		font-size: 1rem;
	}
	.searching {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}
	.searching progress {
		width: 6rem;
	}
</style>
