<!-- Caminho relativo: app/web/src/routes/+page.svelte -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { m } from '#lib/paraglide/messages.js';
	import LanguageSwitch from '#lib/components/LanguageSwitch.svelte';
	import { loadCore, type Core } from '#lib/core.ts';

	const ISSUES_URL = 'https://github.com/psiagoleal/dotree/issues/new';
	const SPEC_URL = 'https://github.com/psiagoleal/dotree/blob/main/docs/specs/README.md';

	type Status =
		{ kind: 'loading' } | { kind: 'ready'; core: Core } | { kind: 'error'; reason: string };

	let status = $state<Status>({ kind: 'loading' });

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
		<p role="status" data-testid="core-status">
			{m.status_ready({ version: status.core.version })}
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
	.notice {
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
