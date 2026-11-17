<!-- Caminho relativo: app/web/src/lib/components/CostSettings.svelte -->
<!-- GER-12 (MT-45): area and delay of FA and HA, in multiples of 0.01, with a button back to
     the convention of spec 00. Costs are a setting, not a help: nothing is revealed or
     counted, and they do not enter the undo history. The metrics, the comparison and the
     solutions follow them; the exercise file keeps them. -->
<script lang="ts">
	import { DEFAULT_COSTS, type Costs } from '#lib/core.ts';
	import { COST_KEYS, costText, isDefaultCosts, parseCost, type CostKey } from '#lib/costs.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';

	let {
		costs,
		onchange,
		readOnly = false
	}: {
		/** The costs in force, from the snapshot. */
		costs: Costs;
		/** Applies new costs; false when the core refused them. */
		onchange: (costs: Costs) => boolean;
		/** Below 768 px (GER-10): shown, not edited. */
		readOnly?: boolean;
	} = $props();

	const id = $props.id();
	const text = (n: number) => costText(n, getLocale());
	const LABEL: Record<CostKey, () => string> = {
		faArea: m.costs_fa_area,
		faDelay: m.costs_fa_delay,
		haArea: m.costs_ha_area,
		haDelay: m.costs_ha_delay
	};

	// The fields start from the costs in force and follow them when they change elsewhere
	// (restore, another exercise); typing changes only the fields until "Apply".
	let fields = $state<Record<CostKey, string>>(fill(DEFAULT_COSTS));
	let refusal = $state<string | null>(null);
	$effect.pre(() => {
		fields = fill(costs);
		refusal = null;
	});

	function fill(c: Costs): Record<CostKey, string> {
		return {
			faArea: text(c.faArea),
			faDelay: text(c.faDelay),
			haArea: text(c.haArea),
			haDelay: text(c.haDelay)
		};
	}

	const parsed = $derived(COST_KEYS.map((k) => [k, parseCost(fields[k])] as const));
	const invalid = $derived(parsed.filter(([, v]) => v === null).map(([k]) => k));
	const summary = $derived(
		m.costs_summary({
			faArea: text(costs.faArea),
			faDelay: text(costs.faDelay),
			haArea: text(costs.haArea),
			haDelay: text(costs.haDelay)
		})
	);

	function apply(e: SubmitEvent) {
		e.preventDefault();
		if (invalid.length > 0) {
			refusal = m.costs_invalid({ fields: invalid.map((k) => LABEL[k]()).join(', ') });
			return;
		}
		const next = Object.fromEntries(parsed) as unknown as Costs;
		refusal = onchange(next)
			? null
			: m.costs_invalid({ fields: COST_KEYS.map((k) => LABEL[k]()).join(', ') });
	}

	function restore() {
		if (onchange({ ...DEFAULT_COSTS })) fields = fill(DEFAULT_COSTS);
	}
</script>

<details class="costs" data-testid="costs">
	<summary>
		{m.costs_title()}
		<span class="current" data-testid="costs-summary">{summary}</span>
		{#if !isDefaultCosts(costs)}<span class="badge">{m.costs_custom()}</span>{/if}
	</summary>
	<form onsubmit={apply} aria-describedby={`${id}-note`}>
		<div class="fields" role="group" aria-label={m.costs_title()}>
			<div class="grid">
				<span></span>
				<span class="head" aria-hidden="true">{m.metric_area()} (A<sub>FA</sub>)</span>
				<span class="head" aria-hidden="true">{m.metric_delay()} (T<sub>FA</sub>)</span>
				<span class="kind" aria-hidden="true">FA</span>
				{#each ['faArea', 'faDelay'] as const as k (k)}
					{@render field(k)}
				{/each}
				<span class="kind" aria-hidden="true">HA</span>
				{#each ['haArea', 'haDelay'] as const as k (k)}
					{@render field(k)}
				{/each}
			</div>
			<div class="buttons">
				<button type="submit" disabled={readOnly}>{m.costs_apply()}</button>
				<button
					type="button"
					class="secondary"
					onclick={restore}
					disabled={readOnly || isDefaultCosts(costs)}>{m.costs_restore()}</button
				>
			</div>
		</div>
		{#if refusal}<p class="refusal" role="alert">{refusal}</p>{/if}
		<p class="note" id={`${id}-note`}>{m.costs_note()}</p>
	</form>
</details>

{#snippet field(k: CostKey)}
	<input
		type="text"
		inputmode="decimal"
		autocomplete="off"
		size="6"
		aria-label={LABEL[k]()}
		aria-invalid={invalid.includes(k) && refusal !== null}
		data-testid={`cost-${k}`}
		disabled={readOnly}
		bind:value={fields[k]}
	/>
{/snippet}

<style>
	summary {
		cursor: pointer;
		color: var(--color-accent);
	}
	.current {
		margin-left: var(--space-2);
		color: var(--color-muted);
		font-size: 0.9rem;
		font-variant-numeric: tabular-nums;
	}
	.badge {
		margin-left: var(--space-2);
		padding: 0 var(--space-1);
		border: 1px solid var(--color-accent);
		border-radius: var(--radius);
		font-size: 0.8rem;
	}
	form {
		display: grid;
		gap: var(--space-2);
		margin-top: var(--space-2);
	}
	.fields {
		display: grid;
		gap: var(--space-2);
		min-width: 0;
	}
	.grid {
		display: grid;
		grid-template-columns: auto minmax(0, 8rem) minmax(0, 8rem);
		gap: var(--space-1) var(--space-2);
		align-items: center;
		justify-content: start;
	}
	.head {
		font-size: 0.85rem;
		color: var(--color-muted);
	}
	.kind {
		font-weight: 600;
	}
	input {
		font: inherit;
		font-variant-numeric: tabular-nums;
		width: 100%;
		box-sizing: border-box;
		padding: var(--space-1) var(--space-2);
		border: 1px solid var(--color-border);
		border-radius: var(--radius);
		background: var(--color-bg);
		color: var(--color-text);
	}
	input[aria-invalid='true'] {
		border-color: var(--color-danger);
	}
	.buttons {
		display: flex;
		flex-wrap: wrap;
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
	.refusal {
		margin: 0;
		color: var(--color-danger);
	}
	.note {
		margin: 0;
		color: var(--color-muted);
		font-size: 0.85rem;
	}
</style>
