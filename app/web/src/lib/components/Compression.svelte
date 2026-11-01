<!-- Caminho relativo: app/web/src/lib/components/Compression.svelte -->
<!-- The compression exercise (spec 02): one read-only panel per concluded level, the current
     level below with selectable points, and the actions, up to the CPA and the result panel.
     Every rule lives in the core; this sends actions to the session and redraws the snapshot
     it returns (ADR 0001). -->
<script lang="ts">
	import { untrack } from 'svelte';
	import type { Cell, CellKind, CompressionSession, Snapshot } from '#lib/core.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import { cpaBlockMessage, rejectionMessage } from '#lib/errors.ts';
	import { keyAction } from '#lib/keys.ts';
	import { formatHalves, visibleTop } from '#lib/levels.ts';
	import { helpLogItems } from '#lib/help.ts';
	import type { ViewMode } from '#lib/view.ts';
	import Coach from './Coach.svelte';
	import ConfirmDialog from './ConfirmDialog.svelte';
	import LevelPanel from './LevelPanel.svelte';
	import Rules from './Rules.svelte';
	import SolutionHelp from './SolutionHelp.svelte';

	let {
		session,
		view,
		outputWidth,
		vMax,
		widthRevealed
	}: {
		session: CompressionSession;
		view: ViewMode;
		/** W and the largest value of the expression; shown only once revealed (EXP-09). */
		outputWidth: number;
		vMax: string;
		widthRevealed: boolean;
	} = $props();

	const id = $props.id();

	// The page re-creates this component for each new session ({#key session}), so the first
	// snapshot is read once here and refreshed after every action.
	let snap = $state.raw<Snapshot>(untrack(() => session.snapshot()));
	let selected = $state<number[]>([]);
	let refusal = $state<string | null>(null);
	let status = $state('');

	let showCritical = $state(false);
	// Points of the next move, highlighted until the next action (AJU-04).
	let suggested = $state<number[]>([]);
	// Read again after each revealed help; the summary shows it (AJU-07).
	let helps = $state.raw(untrack(() => session.helpLog()));
	const helpItems = $derived(helpLogItems(helps));
	let explained = $state(false);
	let backTarget = $state(0);
	let backDialog: ConfirmDialog;
	let whyDialog: ConfirmDialog;

	const complete = $derived(snap.result !== null);
	// A complete circuit names R_0..R_{W-1} (CMP-12), so W is no longer hidden.
	const top = $derived(visibleTop(snap, widthRevealed || complete ? outputWidth : null));
	const critical = $derived(new Set(snap.critical.cells));
	const dropped = $derived(snap.levels.flatMap((l) => l.cells).filter((c) => c.carry === null));
	const KIND: Record<CellKind, string> = { fa: 'FA', ha: 'HA' };
	const halves = (n: number) => formatHalves(n, getLocale());
	const name = (c: Cell) => m.cell_concluded({ kind: KIND[c.kind], number: c.number });

	/** A box's accessible name says what its border and mark show. */
	function label(base: string, isCritical: boolean, isDropped = false): string {
		const text = isDropped ? `${base}, ${m.dropped_carry()}` : base;
		return isCritical ? m.cell_critical({ label: text }) : text;
	}

	function refresh(message: string) {
		snap = session.snapshot();
		selected = [];
		suggested = [];
		refusal = null;
		status = message;
	}

	function toggle(id: number) {
		selected = selected.includes(id) ? selected.filter((s) => s !== id) : [...selected, id];
		refusal = null;
	}

	function form() {
		const r = session.formCell(selected);
		if (r.ok) refresh(m.formed_cell({ kind: KIND[r.kind ?? 'fa'] }));
		else refusal = rejectionMessage(r.rejection);
	}

	function formSuggested(ids: number[]) {
		const r = session.formCell(ids);
		if (r.ok) refresh(m.formed_cell({ kind: KIND[r.kind ?? 'fa'] }));
		else refusal = rejectionMessage(r.rejection);
	}

	function conclude() {
		const level = snap.level;
		const r = session.concludeLevel();
		if (r.ok) refresh(m.level_concluded({ level }));
		else refusal = rejectionMessage(r.rejection);
	}

	function closeWithCpa() {
		const r = session.closeWithCpa();
		if (r.ok) refresh(m.cpa_closed());
		else refusal = rejectionMessage(r.rejection);
	}

	/** CMP-04: asks first whenever going back discards cells (later levels, or the CPA). */
	async function goBack(level: number) {
		const later = snap.levels.slice(level + 1).reduce((n, l) => n + l.cells.length, 0);
		const lost =
			later +
			snap.current.pending.length +
			(snap.levels[level].cpa ? snap.levels[level].cells.length : 0);
		if (lost > 0) {
			backTarget = level;
			if (!(await backDialog.ask())) return;
		}
		const r = session.goToLevel(level);
		if (r.ok) refresh(m.went_back({ level }));
		else refusal = rejectionMessage(r.rejection);
	}

	/** CMP-09: the explanation reveals W, so it asks first while W is hidden. */
	async function explainDropped() {
		if (widthRevealed || (await whyDialog.ask())) explained = true;
	}

	function undo() {
		if (session.undo()) refresh(m.undone());
	}

	function redo() {
		if (session.redo()) refresh(m.redone());
	}

	function clear() {
		selected = [];
		refusal = null;
	}

	function onkeydown(e: KeyboardEvent) {
		const action = keyAction(e);
		if (action === null || action === 'toggleView') return;
		if (document.querySelector('dialog[open]')) return; // a question is being asked
		if (complete && (action === 'form' || action === 'clearSelection')) return;
		e.preventDefault();
		({ form, clearSelection: clear, undo, redo })[action]();
	}
</script>

<svelte:window {onkeydown} />

<section class="compression" aria-labelledby="compression-title">
	<h2 id="compression-title" class="visually-hidden">{m.compress_title()}</h2>

	<div class="metrics-row">
		<p class="metrics" data-testid="metrics" aria-label={m.metrics_title()}>
			<span>FA <b>{snap.metrics.fa}</b></span>
			<span>HA <b>{snap.metrics.ha}</b></span>
			<span>AND <b>{snap.metrics.andGates}</b></span>
			<span>{m.metric_area()} <b>{halves(snap.metrics.area)}</b> A<sub>FA</sub></span>
			<span>{m.metric_delay()} <b>{halves(snap.metrics.delay)}</b> T<sub>FA</sub></span>
		</p>
		<button
			type="button"
			class="secondary"
			aria-pressed={showCritical}
			onclick={() => (showCritical = !showCritical)}>{m.action_critical()}</button
		>
	</div>

	{#each snap.levels as level, i (i)}
		<LevelPanel
			title={level.cpa ? m.level_cpa({ level: i }) : m.level_title({ level: i })}
			testid={`level-${i}`}
			points={level.points}
			groups={level.cells.map((c) => ({
				...c,
				critical: showCritical && critical.has(c.number),
				dropped: c.carry === null,
				label: label(name(c), showCritical && critical.has(c.number), c.carry === null)
			}))}
			{top}
			{view}
		>
			<button type="button" class="link" onclick={() => goBack(i)}>{m.action_back()}</button>
		</LevelPanel>
		<div class="arrow" aria-hidden="true">↓</div>
	{/each}

	<LevelPanel
		title={complete ? m.result_title() : m.level_current({ level: snap.level })}
		testid={complete ? 'result' : 'level-current'}
		points={snap.current.points}
		groups={snap.current.pending.map((c, i) => ({
			...c,
			critical: showCritical && snap.critical.pending.includes(i),
			label: label(
				m.cell_pending({ kind: KIND[c.kind] }),
				showCritical && snap.critical.pending.includes(i)
			)
		}))}
		{top}
		{view}
		heights={!complete}
		selectable={!complete}
		{selected}
		ontoggle={toggle}
		result={snap.result}
		{suggested}
	/>

	{#if dropped.length > 0}
		<div class="dropped">
			<p>
				{m.dropped_note({ cells: dropped.map(name).join(', ') })}
				{#if !explained}
					<button type="button" class="link" onclick={explainDropped}>{m.dropped_why()}</button>
				{/if}
			</p>
			{#if explained}
				<p data-testid="dropped-explanation">
					{m.dropped_explain({ width: outputWidth, top: outputWidth - 1, vmax: vMax })}
				</p>
			{/if}
		</div>
	{/if}

	{#if snap.result}
		<p class="complete" data-testid="complete">
			{m.complete({ width: snap.result.length, top: snap.result.length - 1 })}
		</p>
		<p class="help-log" data-testid="help-log">
			{helpItems.length === 0
				? m.help_log_none()
				: `${m.help_log_title()}: ${helpItems.join(' · ')}`}
		</p>
	{/if}

	<div class="actions">
		{#if !complete}
			<button type="button" onclick={form} disabled={selected.length === 0}
				>{m.action_form()}</button
			>
			<button type="button" class="secondary" onclick={clear} disabled={selected.length === 0}>
				{m.action_clear()}
			</button>
			<span class="count">{m.selection_count({ count: selected.length })}</span>
			<button type="button" class="secondary" onclick={conclude}>{m.action_conclude()}</button>
			<button
				type="button"
				class="secondary"
				onclick={closeWithCpa}
				disabled={snap.cpa !== null}
				aria-describedby={snap.cpa ? `${id}-cpa` : undefined}>{m.action_cpa()}</button
			>
		{/if}
		<button type="button" class="secondary" onclick={undo} disabled={!snap.canUndo}>
			{m.action_undo()}
		</button>
		<button type="button" class="secondary" onclick={redo} disabled={!snap.canRedo}>
			{m.action_redo()}
		</button>
	</div>
	{#if snap.cpa && !complete}
		<p id={`${id}-cpa`} class="cpa-reason">{m.action_cpa()}: {cpaBlockMessage(snap.cpa)}</p>
	{/if}
	{#if refusal}
		<p class="refusal" role="alert">{refusal}</p>
	{/if}
	<p class="visually-hidden" role="status">{status}</p>
	<p class="hint">{m.keyboard_hint()}</p>

	<section class="help" aria-labelledby={`${id}-help`}>
		<h3 id={`${id}-help`}>{m.help_title()}</h3>
		<Rules />
		{#if !complete}
			<!-- Re-created on every change, so an answer about an older state never stays. -->
			{#key snap}
				<Coach
					{session}
					points={snap.current.points}
					onsuggest={(ids) => (suggested = ids)}
					onform={formSuggested}
					onhelp={() => (helps = session.helpLog())}
				/>
			{/key}
		{/if}
		<SolutionHelp {session} {view} onhelp={() => (helps = session.helpLog())} />
	</section>

	<ConfirmDialog
		bind:this={backDialog}
		title={m.back_confirm_title({ level: backTarget })}
		body={m.back_confirm_body({ level: backTarget })}
		cancelLabel={m.back_confirm_cancel()}
		confirmLabel={m.back_confirm_ok()}
	/>
	<ConfirmDialog
		bind:this={whyDialog}
		title={m.dropped_confirm_title()}
		body={m.dropped_confirm_body()}
	/>
</section>

<style>
	.compression {
		display: grid;
		gap: var(--space-2);
	}
	.metrics-row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
	}
	.metrics {
		flex: 1;
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1) var(--space-3);
		margin: 0;
		padding: var(--space-2) var(--space-3);
		border: 1px solid var(--color-border);
		border-radius: var(--radius);
		background: var(--color-surface);
		font-variant-numeric: tabular-nums;
	}
	.arrow {
		color: var(--color-muted);
		font-size: 1.25rem;
		padding-left: var(--space-3);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
	}
	button {
		font: inherit;
		padding: var(--space-1) var(--space-3);
		border-radius: var(--radius);
		border: 1px solid var(--color-accent);
		background: var(--color-accent);
		color: var(--color-bg);
		cursor: pointer;
	}
	button.secondary {
		background: transparent;
		color: var(--color-accent);
	}
	button.link {
		padding: 0;
		border: 0;
		background: none;
		color: var(--color-accent);
		text-decoration: underline;
		font-size: 0.9rem;
	}
	button[aria-pressed='true'] {
		background: var(--color-accent);
		color: var(--color-bg);
	}
	button:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}
	.count {
		color: var(--color-muted);
		font-variant-numeric: tabular-nums;
	}
	.cpa-reason,
	.dropped p {
		margin: 0;
		color: var(--color-muted);
		font-size: 0.9rem;
	}
	.help {
		display: grid;
		gap: var(--space-2);
		margin-top: var(--space-3);
		padding-top: var(--space-3);
		border-top: 1px solid var(--color-border);
	}
	.help h3 {
		margin: 0;
		font-size: 1rem;
	}
	.help-log {
		margin: 0;
		color: var(--color-muted);
	}
	.complete {
		margin: 0;
		font-weight: 600;
		color: var(--dot-result);
	}
	.refusal {
		margin: 0;
		color: var(--color-danger);
	}
	.hint {
		margin: 0;
		color: var(--color-muted);
		font-size: 0.85rem;
	}
	.visually-hidden {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip-path: inset(50%);
		white-space: nowrap;
	}
</style>
