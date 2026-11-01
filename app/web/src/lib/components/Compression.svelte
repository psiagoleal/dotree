<!-- Caminho relativo: app/web/src/lib/components/Compression.svelte -->
<!-- The compression exercise (spec 02): one read-only panel per concluded level, the current
     level below with selectable points, and the actions. Every rule lives in the core; this
     sends actions to the session and redraws the snapshot it returns (ADR 0001). -->
<script lang="ts">
	import { untrack } from 'svelte';
	import type { CellKind, CompressionSession, Snapshot } from '#lib/core.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import { rejectionMessage } from '#lib/errors.ts';
	import { keyAction } from '#lib/keys.ts';
	import { formatHalves, visibleTop } from '#lib/levels.ts';
	import type { ViewMode } from '#lib/view.ts';
	import LevelPanel from './LevelPanel.svelte';

	let {
		session,
		view,
		width
	}: { session: CompressionSession; view: ViewMode; width: number | null } = $props();

	// The page re-creates this component for each new session ({#key session}), so the first
	// snapshot is read once here and refreshed after every action.
	let snap = $state.raw<Snapshot>(untrack(() => session.snapshot()));
	let selected = $state<number[]>([]);
	let refusal = $state<string | null>(null);
	let status = $state('');

	const top = $derived(visibleTop(snap, width));
	const KIND: Record<CellKind, string> = { fa: 'FA', ha: 'HA' };
	const halves = (n: number) => formatHalves(n, getLocale());

	function refresh(message: string) {
		snap = session.snapshot();
		selected = [];
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

	function conclude() {
		const level = snap.level;
		const r = session.concludeLevel();
		if (r.ok) refresh(m.level_concluded({ level }));
		else refusal = rejectionMessage(r.rejection);
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
		e.preventDefault();
		({ form, clearSelection: clear, undo, redo })[action]();
	}
</script>

<svelte:window {onkeydown} />

<section class="compression" aria-labelledby="compression-title">
	<h2 id="compression-title" class="visually-hidden">{m.compress_title()}</h2>

	<p class="metrics" data-testid="metrics" aria-label={m.metrics_title()}>
		<span>FA <b>{snap.metrics.fa}</b></span>
		<span>HA <b>{snap.metrics.ha}</b></span>
		<span>AND <b>{snap.metrics.andGates}</b></span>
		<span>{m.metric_area()} <b>{halves(snap.metrics.area)}</b> A<sub>FA</sub></span>
		<span>{m.metric_delay()} <b>{halves(snap.metrics.delay)}</b> T<sub>FA</sub></span>
	</p>

	{#each snap.levels as level, i (i)}
		<LevelPanel
			title={m.level_title({ level: i })}
			testid={`level-${i}`}
			points={level.points}
			groups={level.cells.map((c) => ({
				...c,
				label: m.cell_concluded({ kind: KIND[c.kind], number: c.number })
			}))}
			{top}
			{view}
		/>
		<div class="arrow" aria-hidden="true">↓</div>
	{/each}

	<LevelPanel
		title={m.level_current({ level: snap.level })}
		testid="level-current"
		points={snap.current.points}
		groups={snap.current.pending.map((c) => ({
			...c,
			label: m.cell_pending({ kind: KIND[c.kind] })
		}))}
		{top}
		{view}
		heights
		selectable
		{selected}
		ontoggle={toggle}
	/>

	<div class="actions">
		<button type="button" onclick={form} disabled={selected.length === 0}>{m.action_form()}</button>
		<button type="button" class="secondary" onclick={clear} disabled={selected.length === 0}>
			{m.action_clear()}
		</button>
		<span class="count">{m.selection_count({ count: selected.length })}</span>
		<button type="button" class="secondary" onclick={conclude}>{m.action_conclude()}</button>
		<button type="button" class="secondary" onclick={undo} disabled={!snap.canUndo}>
			{m.action_undo()}
		</button>
		<button type="button" class="secondary" onclick={redo} disabled={!snap.canRedo}>
			{m.action_redo()}
		</button>
	</div>
	{#if refusal}
		<p class="refusal" role="alert">{refusal}</p>
	{/if}
	<p class="visually-hidden" role="status">{status}</p>
	<p class="hint">{m.keyboard_hint()}</p>
</section>

<style>
	.compression {
		display: grid;
		gap: var(--space-2);
	}
	.metrics {
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
	button:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}
	.count {
		color: var(--color-muted);
		font-variant-numeric: tabular-nums;
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
