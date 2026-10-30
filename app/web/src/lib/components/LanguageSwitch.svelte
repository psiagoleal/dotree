<!-- Caminho relativo: app/web/src/lib/components/LanguageSwitch.svelte -->
<script lang="ts">
	import { getLocale, setLocale } from '#lib/paraglide/runtime.js';
	import { m } from '#lib/paraglide/messages.js';
	import type { AppLocale } from '#lib/locale.ts';

	let current = $state(getLocale() as AppLocale);

	const options: { locale: AppLocale; label: () => string }[] = [
		{ locale: 'pt-br', label: m.language_pt_br },
		{ locale: 'en', label: m.language_en }
	];

	function choose(locale: AppLocale) {
		// Paraglide re-renders through a reload by default; the app state will survive it once
		// SAL-01 (local autosave) exists. Until then a reload is harmless.
		current = locale;
		setLocale(locale);
	}
</script>

<div class="switch" role="group" aria-label={m.language_label()}>
	{#each options as option (option.locale)}
		<button
			type="button"
			aria-pressed={current === option.locale}
			lang={option.locale === 'pt-br' ? 'pt-BR' : 'en'}
			onclick={() => choose(option.locale)}
		>
			{option.label()}
		</button>
	{/each}
</div>

<style>
	.switch {
		display: inline-flex;
		gap: var(--space-1);
	}
	button {
		font: inherit;
		padding: var(--space-1) var(--space-2);
		border: 1px solid var(--color-border);
		border-radius: var(--radius);
		background: var(--color-surface);
		color: var(--color-text);
		cursor: pointer;
	}
	button[aria-pressed='true'] {
		border-color: var(--color-accent);
		color: var(--color-accent);
		font-weight: 600;
	}
</style>
