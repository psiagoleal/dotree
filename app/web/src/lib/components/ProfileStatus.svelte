<!-- Caminho relativo: app/web/src/lib/components/ProfileStatus.svelte -->
<!-- The output profile of a height profile exercise (CMP-19): what was asked for, whether
     the student's circuit reached it and, then, its area and delay. The name of the known
     block is a counted help on request (AJU-12), shown again uncounted once revealed. -->
<script lang="ts">
	import { untrack } from 'svelte';
	import type { Block, CompressionSession, Snapshot } from '#lib/core.ts';
	import { formatHalves } from '#lib/levels.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import ConfirmDialog from './ConfirmDialog.svelte';

	let {
		session,
		snap,
		onhelp,
		profile
	}: {
		session: CompressionSession;
		snap: Snapshot;
		onhelp: () => void;
		profile: { output: number[]; reached: boolean };
	} = $props();

	let blockDialog: ConfirmDialog;
	let block = $state.raw<Block | null>(untrack(() => session.shownBlock()));

	const halves = (n: number) => formatHalves(n, getLocale());
	/** The profile as typed: most significant column first. */
	const asked = $derived(`{${[...profile.output].reverse().join(', ')}}`);
	const target = (column: number) => profile.output[column] ?? 0;
	/** Columns above their target, most significant first. */
	const over = $derived(
		snap.heights
			.map((height, column) => ({ column, height, target: target(column) }))
			.filter((c) => c.height > c.target)
			.reverse()
	);

	function blockName(b: Block): string {
		switch (b.code) {
			case 'fullAdder':
				return m.block_full_adder();
			case 'halfAdder':
				return m.block_half_adder();
			case 'adder':
				return m.block_adder({ bits: b.bits });
			case 'adderWithCarryIn':
				return m.block_adder_carry_in({ bits: b.bits });
			case 'carrySave':
				return m.block_carry_save({ bits: b.bits });
			case 'counter':
				return m.block_counter();
			case 'compressor':
				return m.block_compressor();
		}
	}

	async function askBlock() {
		if (!(await blockDialog.ask())) return;
		block = session.block();
		onhelp();
	}
</script>

<section class="profile" aria-labelledby="profile-title" data-testid="profile">
	<h3 id="profile-title">{m.profile_title()}</h3>
	<p>{m.profile_target({ profile: asked })}</p>
	{#if profile.reached}
		<p class="reached" role="status" data-testid="profile-reached">
			{m.profile_reached({
				area: halves(snap.metrics.area),
				delay: halves(snap.metrics.delay)
			})}
		</p>
	{:else if over.length > 0}
		<p data-testid="profile-over">
			{m.profile_over({
				columns: over.map((c) => m.profile_over_column({ ...c })).join('; ')
			})}
		</p>
	{:else}
		<p data-testid="profile-pending">{m.profile_pending()}</p>
	{/if}
	{#if block}
		<p data-testid="profile-block">{blockName(block)}</p>
	{:else}
		<div>
			<button type="button" class="secondary" onclick={askBlock}>{m.profile_block_ask()}</button>
		</div>
	{/if}
</section>

<ConfirmDialog
	bind:this={blockDialog}
	title={m.profile_block_confirm_title()}
	body={m.profile_block_confirm_body()}
/>

<style>
	.profile {
		display: grid;
		gap: var(--space-1);
		padding: var(--space-2) var(--space-3);
		border: 1px solid var(--color-border);
		border-radius: var(--radius);
	}
	h3 {
		margin: 0;
		font-size: 1rem;
	}
	p {
		margin: 0;
	}
	.reached {
		font-weight: 600;
	}
	button {
		font: inherit;
		padding: var(--space-1) var(--space-3);
		border-radius: var(--radius);
		border: 1px solid var(--color-accent);
		background: transparent;
		color: var(--color-accent);
		cursor: pointer;
	}
</style>
