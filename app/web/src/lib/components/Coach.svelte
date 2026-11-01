<!-- Caminho relativo: app/web/src/lib/components/Coach.svelte -->
<!-- Hint and next move (spec 03): both are about the current state, so the parent re-creates
     this component on every change and a stale answer never stays on screen. Each request
     asks first (AJU-02) and each confirmation reveals once (D-5); the core counts it. -->
<script lang="ts">
	import type { CompressionSession, LevelPoint, NextMove } from '#lib/core.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { hintMessage, moveMessage } from '#lib/help.ts';
	import ConfirmDialog from './ConfirmDialog.svelte';

	let {
		session,
		points,
		onsuggest,
		onform,
		onhelp
	}: {
		session: CompressionSession;
		/** Points of the current level, to name the suggested cell. */
		points: LevelPoint[];
		/** Highlight these points in the matrix (AJU-04). */
		onsuggest: (ids: number[]) => void;
		/** The user asked to form the suggested cell. */
		onform: (ids: number[]) => void;
		/** A help was revealed (AJU-07). */
		onhelp: () => void;
	} = $props();

	let hintDialog: ConfirmDialog;
	let moveDialog: ConfirmDialog;
	let hint = $state<string | null>(null);
	let move = $state.raw<NextMove | null>(null);

	async function askHint() {
		if (!(await hintDialog.ask())) return;
		const h = session.hint();
		hint = h ? hintMessage(h) : m.move_complete();
		onhelp();
	}

	async function askMove() {
		if (!(await moveDialog.ask())) return;
		move = session.nextMove();
		if (move.move === 'formCell') onsuggest(move.points);
		onhelp();
	}
</script>

<div class="coach">
	<div class="buttons">
		<button type="button" class="secondary" onclick={askHint}>{m.help_hint()}</button>
		<button type="button" class="secondary" onclick={askMove}>{m.help_next_move()}</button>
	</div>
	{#if hint}
		<p data-testid="hint">{hint}</p>
	{/if}
	{#if move}
		<p data-testid="next-move">
			{moveMessage(move, points)}
			{#if move.move === 'formCell'}
				{@const ids = move.points}
				<button type="button" class="link" onclick={() => onform(ids)}>{m.move_form_this()}</button>
			{/if}
		</p>
	{/if}
</div>

<ConfirmDialog bind:this={hintDialog} title={m.confirm_hint_title()} body={m.confirm_hint_body()} />
<ConfirmDialog
	bind:this={moveDialog}
	title={m.confirm_next_move_title()}
	body={m.confirm_next_move_body()}
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
	button.link {
		padding: 0;
		border: 0;
		text-decoration: underline;
		font-size: 0.9rem;
	}
	.coach {
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
