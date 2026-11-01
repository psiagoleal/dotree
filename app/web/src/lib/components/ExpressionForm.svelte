<!-- Caminho relativo: app/web/src/lib/components/ExpressionForm.svelte -->
<!-- Expression and one width per upper-case letter in it, plus the built-in examples
     (EXP-01, EXP-12). Validation is the core's job; this only collects the input. -->
<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import { DEFAULT_WIDTH, EXAMPLES, variablesIn, type Example } from '#lib/expression.ts';

	let {
		onbuild
	}: {
		onbuild: (expression: string, widths: Record<string, number>) => void;
	} = $props();

	let expression = $state('');
	let widths = $state<Record<string, number>>({});
	const vars = $derived(variablesIn(expression));

	const exampleName: Record<Example['id'], () => string> = {
		r1: m.example_r1,
		r3: m.example_r3,
		r4: m.example_r4
	};

	function widthsInUse(): Record<string, number> {
		return Object.fromEntries(vars.map((v) => [v, widths[v] ?? DEFAULT_WIDTH]));
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		onbuild(expression, widthsInUse());
	}

	function useExample(ex: Example) {
		expression = ex.expression;
		widths = { ...ex.widths };
		onbuild(ex.expression, { ...ex.widths });
	}
</script>

<form onsubmit={submit} class="form">
	<label class="expr">
		<span>{m.form_expression_label()}</span>
		<input
			name="expression"
			bind:value={expression}
			autocomplete="off"
			spellcheck="false"
			aria-describedby="expr-hint"
		/>
	</label>
	<p id="expr-hint" class="hint">{m.form_expression_hint()}</p>

	{#if vars.length > 0}
		<fieldset class="widths">
			<legend>{m.form_widths_label()}</legend>
			{#each vars as v (v)}
				<label>
					<span>{v}</span>
					<input
						type="number"
						min="1"
						max="16"
						name={`width-${v}`}
						value={widths[v] ?? DEFAULT_WIDTH}
						oninput={(e) => (widths[v] = Number(e.currentTarget.value))}
					/>
				</label>
			{/each}
		</fieldset>
	{/if}

	<div class="actions">
		<button type="submit">{m.form_build()}</button>
		<span class="examples-label">{m.form_examples()}:</span>
		{#each EXAMPLES as ex (ex.id)}
			<button type="button" class="example" onclick={() => useExample(ex)}>
				{exampleName[ex.id]()}
			</button>
		{/each}
	</div>
</form>

<style>
	.form {
		display: grid;
		gap: var(--space-2);
	}
	.expr {
		display: grid;
		gap: var(--space-1);
		font-weight: 600;
	}
	input {
		font: inherit;
		padding: var(--space-1) var(--space-2);
		border: 1px solid var(--color-border);
		border-radius: var(--radius);
		background: var(--color-surface);
		color: var(--color-text);
	}
	.expr input {
		font-family: var(--font-mono);
		font-weight: 400;
	}
	.hint {
		margin: 0;
		color: var(--color-muted);
		font-size: 0.9rem;
	}
	.widths {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		border: 1px solid var(--color-border);
		border-radius: var(--radius);
	}
	.widths label {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		font-family: var(--font-mono);
	}
	.widths input {
		width: 4.5rem;
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
	button.example {
		background: var(--color-surface);
		color: var(--color-accent);
	}
	.examples-label {
		color: var(--color-muted);
	}
</style>
