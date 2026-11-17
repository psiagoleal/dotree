<!-- Caminho relativo: app/web/src/lib/components/CostSettings.svelte -->
<!-- GER-12 (MT-45): area and delay of FA and HA, in multiples of 0.01, with a button back to
     the convention of spec 00. Costs are a setting, not a help: nothing is revealed or
     counted, and they do not enter the undo history. The metrics, the comparison and the
     solutions follow them; the exercise file keeps them. -->
<script lang="ts">
	import { DEFAULT_COSTS, type Costs } from '#lib/core.ts';
	import {
		COST_KEYS,
		COUNTER_KEYS,
		costText,
		counterCost,
		isDefaultCosts,
		networkCost,
		parseCost,
		type CostKey,
		type CounterKey
	} from '#lib/costs.ts';
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
	type Key = CostKey | CounterKey;
	const LABEL: Record<Key, () => string> = {
		faArea: m.costs_fa_area,
		faDelay: m.costs_fa_delay,
		haArea: m.costs_ha_area,
		haDelay: m.costs_ha_delay,
		c73Area: m.costs_c73_area,
		c73Delay: m.costs_c73_delay,
		c154Area: m.costs_c154_area,
		c154Delay: m.costs_c154_delay
	};
	const isCounterKey = (k: Key): k is CounterKey => (COUNTER_KEYS as readonly string[]).includes(k);

	// The fields start from the costs in force and follow them when they change elsewhere
	// (restore, another exercise); typing changes only the fields until "Apply".
	let fields = $state<Record<Key, string>>(fill(DEFAULT_COSTS));
	let refusal = $state<string | null>(null);
	$effect.pre(() => {
		fields = fill(costs);
		refusal = null;
	});

	/** A counter field is blank while the counter follows its FA network. */
	function fill(c: Costs): Record<Key, string> {
		const own = (k: CounterKey) => (c[k] === undefined ? '' : text(c[k]));
		return {
			faArea: text(c.faArea),
			faDelay: text(c.faDelay),
			haArea: text(c.haArea),
			haDelay: text(c.haDelay),
			c73Area: own('c73Area'),
			c73Delay: own('c73Delay'),
			c154Area: own('c154Area'),
			c154Delay: own('c154Delay')
		};
	}

	const parsed = $derived(COST_KEYS.map((k) => [k, parseCost(fields[k])] as const));
	// Blank counter fields are valid: no cost of their own.
	const parsedCounters = $derived(
		COUNTER_KEYS.filter((k) => fields[k].trim() !== '').map(
			(k) => [k, parseCost(fields[k])] as const
		)
	);
	const invalid = $derived(
		[...parsed, ...parsedCounters].filter(([, v]) => v === null).map(([k]) => k as Key)
	);
	const summary = $derived(
		[
			m.costs_summary({
				faArea: text(costs.faArea),
				faDelay: text(costs.faDelay),
				haArea: text(costs.haArea),
				haDelay: text(costs.haDelay)
			}),
			// A counter is named only when it has a cost of its own.
			...(
				[
					['7:3', 'c73Area', 'c73Delay'],
					['15:4', 'c154Area', 'c154Delay']
				] as const
			)
				.filter(([, a, d]) => costs[a] !== undefined || costs[d] !== undefined)
				.map(([kind, a, d]) =>
					m.costs_summary_counter({
						kind,
						area: text(counterCost(costs, a)),
						delay: text(counterCost(costs, d))
					})
				)
		].join(' · ')
	);

	function apply(e: SubmitEvent) {
		e.preventDefault();
		if (invalid.length > 0) {
			refusal = m.costs_invalid({ fields: invalid.map((k) => LABEL[k]()).join(', ') });
			return;
		}
		const next = Object.fromEntries([...parsed, ...parsedCounters]) as unknown as Costs;
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
				<span class="kind" aria-hidden="true">7:3</span>
				{#each ['c73Area', 'c73Delay'] as const as k (k)}
					{@render field(k)}
				{/each}
				<span class="kind" aria-hidden="true">15:4</span>
				{#each ['c154Area', 'c154Delay'] as const as k (k)}
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

{#snippet field(k: Key)}
	<input
		placeholder={isCounterKey(k)
			? m.costs_network({ value: text(networkCost(costs, k)) })
			: undefined}
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
