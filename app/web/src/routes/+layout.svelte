<!-- Caminho relativo: app/web/src/routes/+layout.svelte -->
<script lang="ts">
	import '../app.css';
	import favicon from '#lib/assets/favicon.svg';
	import { getLocale, setLocale } from '#lib/paraglide/runtime.js';
	import { detectLocale, hasStoredLocale } from '#lib/locale.ts';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();

	// First visit: pick the language from the browser (IDI-03). Later visits: Paraglide
	// restores the stored choice (IDI-04).
	if (typeof window !== 'undefined' && !hasStoredLocale(window.localStorage)) {
		const detected = detectLocale(navigator.languages ?? [navigator.language]);
		if (detected !== getLocale()) setLocale(detected, { reload: false });
	}

	$effect(() => {
		document.documentElement.lang = getLocale() === 'pt-br' ? 'pt-BR' : 'en';
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}
