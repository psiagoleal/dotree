<!-- Caminho relativo: app/web/src/lib/components/WidthChallenge.svelte -->
<!-- "How many bits does the output need?" (D-1). W stays hidden until the user answers or
     asks for it and confirms (EXP-09, EXP-10, GER-01). The working, from v_max to W, is
     step 2, with its own request and confirmation (GER-03). -->
<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import type { Matrix } from '#lib/core.ts';
	import { widthPath } from '#lib/width.ts';
	import ConfirmDialog from './ConfirmDialog.svelte';

	let { matrix, onreveal }: { matrix: Matrix; onreveal: () => void } = $props();

	type Outcome = { kind: 'right' } | { kind: 'wrong'; answer: number } | { kind: 'given-up' };

	let draft = $state('');
	let invalid = $state(false);
	let outcome = $state<Outcome | null>(null);
	let showPath = $state(false);
	let resultDialog: ConfirmDialog;
	let pathDialog: ConfirmDialog;

	const path = $derived(widthPath(matrix));
	const width = $derived(matrix.outputWidth);

	function reveal(o: Outcome) {
		outcome = o;
		onreveal();
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		const text = draft.trim();
		const answer = Number(text);
		invalid = !/^\d+$/.test(text) || answer < 1;
		if (invalid) return;
		reveal(answer === width ? { kind: 'right' } : { kind: 'wrong', answer });
	}

	async function giveUp() {
		if (await resultDialog.ask()) reveal({ kind: 'given-up' });
	}

	async function askPath() {
		if (await pathDialog.ask()) showPath = true;
	}
</script>

<section class="challenge" aria-labelledby="width-title">
	<h3 id="width-title">{m.width_title()}</h3>

	{#if outcome === null}
		<p>{m.width_question()}</p>
		<form onsubmit={submit} novalidate>
			<label>
				<span>{m.width_answer_label()}</span>
				<input
					name="width-answer"
					inputmode="numeric"
					autocomplete="off"
					bind:value={draft}
					aria-invalid={invalid}
					aria-describedby={invalid ? 'width-invalid' : undefined}
				/>
			</label>
			<button type="submit">{m.width_answer_submit()}</button>
			<button type="button" class="secondary" onclick={giveUp}>{m.width_give_up()}</button>
		</form>
		{#if invalid}
			<p id="width-invalid" class="invalid" data-testid="width-invalid" role="alert">
				{m.width_answer_invalid()}
			</p>
		{/if}
	{:else}
		<p class="result" role="status" data-testid="width-result">
			{#if outcome.kind === 'right'}
				{m.width_right({ width })}
			{:else if outcome.kind === 'wrong'}
				{m.width_wrong({ answer: outcome.answer, width })}
			{:else}
				{m.width_revealed({ width })}
			{/if}
		</p>

		{#if showPath}
			<div class="path" data-testid="width-path">
				<p>{m.width_path_inputs()}</p>
				<ul class="math inputs">
					{#each path.inputs as i (i.var)}
						<li>{i.var} = 2<sup>{i.width}</sup> − 1 = {i.max}</li>
					{/each}
				</ul>
				<p>{m.width_path_vmax()}</p>
				<p class="math">
					v<sub>max</sub> = {path.symbolic}<br />
					= {path.substituted}<br />
					{#if path.termValues}= {path.termValues}<br />{/if}
					= {path.vMax}
				</p>
				<p>{m.width_path_w()}</p>
				<p class="math">
					W = ⌈log<sub>2</sub>(v<sub>max</sub> + 1)⌉ = ⌈log<sub>2</sub>
					{path.vMaxPlusOne}⌉ = {width},
					{m.width_path_because()}
					2<sup>{path.lower.exponent}</sup> = {path.lower.value} ≤ {path.vMax} &lt; {path.upper
						.value} = 2<sup>{path.upper.exponent}</sup>
				</p>
			</div>
		{:else}
			<button type="button" class="secondary" onclick={askPath}>{m.width_show_path()}</button>
		{/if}
	{/if}
</section>

<ConfirmDialog
	bind:this={resultDialog}
	title={m.width_confirm_result_title()}
	body={m.width_confirm_result_body()}
/>
<ConfirmDialog
	bind:this={pathDialog}
	title={m.width_confirm_path_title()}
	body={m.width_confirm_path_body()}
/>

<style>
	.challenge {
		display: grid;
		gap: var(--space-2);
		justify-items: start;
		padding: var(--space-3);
		border: 1px solid var(--color-border);
		border-radius: var(--radius);
		background: var(--color-surface);
	}
	h3 {
		margin: 0;
		font-size: 1rem;
	}
	p {
		margin: 0;
	}
	form {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
	}
	label {
		display: flex;
		align-items: center;
		gap: var(--space-1);
	}
	input {
		width: 5rem;
		font: inherit;
		font-family: var(--font-mono);
		padding: var(--space-1) var(--space-2);
		border: 1px solid var(--color-border);
		border-radius: var(--radius);
		background: var(--color-bg);
		color: var(--color-text);
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
	.invalid {
		color: var(--color-danger);
	}
	.result {
		font-weight: 600;
	}
	.path {
		display: grid;
		gap: var(--space-1);
	}
	.inputs {
		display: flex;
		flex-wrap: wrap;
		gap: 0 var(--space-3);
		margin: 0;
		list-style: none;
	}
	.inputs li {
		white-space: nowrap;
	}
	.math {
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
		padding-left: var(--space-3);
		line-height: 1.6;
	}
</style>
