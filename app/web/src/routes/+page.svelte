<!-- Caminho relativo: app/web/src/routes/+page.svelte -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { m } from '#lib/paraglide/messages.js';
	import LanguageSwitch from '#lib/components/LanguageSwitch.svelte';
	import ExpressionForm from '#lib/components/ExpressionForm.svelte';
	import DotMatrix from '#lib/components/DotMatrix.svelte';
	import TermLegend from '#lib/components/TermLegend.svelte';
	import { loadCore, type BuildResult, type Core } from '#lib/core.ts';
	import { errorMessage, warningMessage } from '#lib/errors.ts';

	const ISSUES_URL = 'https://github.com/psiagoleal/dotree/issues/new';
	const SPEC_URL = 'https://github.com/psiagoleal/dotree/blob/main/docs/specs/README.md';

	type Status =
		{ kind: 'loading' } | { kind: 'ready'; core: Core } | { kind: 'error'; reason: string };

	let status = $state<Status>({ kind: 'loading' });
	let result = $state<BuildResult | null>(null);

	function build(core: Core, expression: string, widths: Record<string, number>) {
		result = core.buildMatrix(expression, widths);
	}

	onMount(() => {
		loadCore()
			.then((core) => (status = { kind: 'ready', core }))
			.catch((err: unknown) => {
				// GER-09: a broken WASM load gets an explanation, not a broken interface.
				status = { kind: 'error', reason: err instanceof Error ? err.message : String(err) };
			});
	});
</script>

<header>
	<h1>{m.app_title()}</h1>
	<LanguageSwitch />
</header>

<main>
	<p class="tagline">{m.app_tagline()}</p>

	{#if status.kind === 'loading'}
		<p role="status">{m.status_loading()}</p>
	{:else if status.kind === 'ready'}
		{@const core = status.core}
		<ExpressionForm onbuild={(e, w) => build(core, e, w)} />

		{#if result && !result.ok}
			<p role="alert" class="error" data-testid="build-error">{errorMessage(result.error)}</p>
		{:else if result && result.ok}
			{@const mx = result.matrix}
			<section class="result" aria-labelledby="matrix-title">
				<h2 id="matrix-title">{m.matrix_title()}</h2>
				{#each mx.warnings as w, i (i)}
					<p class="warning" role="status">{warningMessage(w)}</p>
				{/each}
				<p class="summary" data-testid="matrix-summary">
					{m.matrix_summary({ dots: mx.dots.length, ands: mx.andGates })}
				</p>
				<DotMatrix matrix={mx} />
				<TermLegend terms={mx.terms} />
			</section>
		{/if}

		<p class="version" data-testid="core-status">
			{m.status_ready({ version: core.version })}
		</p>
	{:else}
		<section role="alert" class="error" data-testid="core-error">
			<h2>{m.status_error_title()}</h2>
			<p>{m.status_error_reason({ reason: status.reason })}</p>
			<p><a href={ISSUES_URL}>{m.status_error_report()}</a></p>
		</section>
	{/if}

	<p class="notice">{m.wip_notice()} <a href={SPEC_URL}>{m.spec_link()}</a></p>
</main>

<style>
	header,
	main {
		max-width: 60rem;
		margin: 0 auto;
		padding: var(--space-3);
	}
	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
		border-bottom: 1px solid var(--color-border);
	}
	h1 {
		margin: 0;
		font-family: var(--font-mono);
		letter-spacing: 0.02em;
	}
	.tagline {
		font-size: 1.15rem;
	}
	.notice,
	.version {
		color: var(--color-muted);
		font-size: 0.9rem;
	}
	.result {
		display: grid;
		gap: var(--space-2);
		margin-top: var(--space-4);
	}
	.result h2 {
		margin: 0;
		font-size: 1.2rem;
	}
	.summary {
		margin: 0;
		color: var(--color-muted);
		font-variant-numeric: tabular-nums;
	}
	p.error {
		color: var(--color-danger);
		margin-top: var(--space-3);
	}
	.warning {
		margin: 0;
		color: var(--color-muted);
	}
	.error {
		border: 1px solid var(--color-danger);
		border-radius: var(--radius);
		padding: var(--space-3);
	}
	.error h2 {
		margin-top: 0;
		color: var(--color-danger);
		font-size: 1.1rem;
	}
</style>
