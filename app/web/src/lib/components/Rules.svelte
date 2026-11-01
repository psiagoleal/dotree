<!-- Caminho relativo: app/web/src/lib/components/Rules.svelte -->
<!-- The notation reminder (AJU-01): it says nothing about the current exercise, so it is
     always available and asks nothing. -->
<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
</script>

<details>
	<summary>{m.rules_title()}</summary>
	<ul>
		<li>{m.rules_fa()}</li>
		<li>{m.rules_ha()}</li>
		<li>{m.rules_level()}</li>
		<li>{m.rules_cpa()}</li>
		<li>{m.rules_done()}</li>
	</ul>
</details>

<style>
	summary {
		cursor: pointer;
		color: var(--color-accent);
	}
	ul {
		margin: var(--space-2) 0 0;
		padding-left: var(--space-4);
		display: grid;
		gap: var(--space-1);
		font-size: 0.9rem;
	}
</style>
