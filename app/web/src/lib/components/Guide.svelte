<!-- Caminho relativo: app/web/src/lib/components/Guide.svelte -->
<!-- "How to use" (GER-11): opened from the header at any time, with no confirmation and no
     count, because nothing in it depends on the open exercise. Escape or "Close" shuts it,
     and the focus goes back to the button that opened it. -->
<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import { guideSections } from '#lib/guide.ts';

	const REPOSITORY_URL = 'https://github.com/psiagoleal/dotree#readme';

	const id = $props.id();
	let dialog: HTMLDialogElement;
	let opener: HTMLButtonElement;
	let open = $state(false);
	// Built on opening, in the language of the moment.
	const sections = $derived(open ? guideSections() : []);

	function show() {
		open = true;
		dialog.showModal();
	}

	function onclose() {
		open = false;
		opener.focus();
	}
</script>

<button bind:this={opener} type="button" class="open" aria-haspopup="dialog" onclick={show}>
	{m.guide_open()}
</button>

<dialog bind:this={dialog} aria-labelledby={`${id}-title`} {onclose}>
	{#if open}
		<div class="head">
			<h2 id={`${id}-title`}>{m.guide_title()}</h2>
			<button type="button" class="close" onclick={() => dialog.close()}>{m.guide_close()}</button>
		</div>
		<p>{m.guide_intro()}</p>
		{#each sections as section (section.id)}
			<section aria-labelledby={`${id}-${section.id}`}>
				<h3 id={`${id}-${section.id}`}>{section.title}</h3>
				{#if section.intro}<p>{section.intro}</p>{/if}
				{#if section.id === 'steps'}
					<ol>
						{#each section.items as item, i (i)}<li>{item.text}</li>{/each}
					</ol>
				{:else}
					<ul class:keys={section.id === 'keys'}>
						{#each section.items as item, i (i)}
							<li>
								{#if item.keys}
									<span class="kbds"
										>{#each item.keys as key, k (key)}{#if k > 0}
												/
											{/if}<kbd>{key}</kbd>{/each}</span
									>
								{/if}
								{item.text}
							</li>
						{/each}
					</ul>
				{/if}
				{#if section.id === 'references'}
					<p><a href={REPOSITORY_URL}>{m.guide_ref_more()}</a></p>
				{/if}
			</section>
		{/each}
	{/if}
</dialog>

<style>
	button {
		font: inherit;
		padding: var(--space-1) var(--space-3);
		border-radius: var(--radius);
		border: 1px solid var(--color-accent);
		background: transparent;
		color: var(--color-accent);
		cursor: pointer;
	}
	dialog {
		width: min(44rem, calc(100vw - 2rem));
		max-height: calc(100vh - 2rem);
		overflow: auto;
		padding: var(--space-4);
		border: 1px solid var(--color-border);
		border-radius: var(--radius);
		background: var(--color-surface);
		color: var(--color-text);
	}
	dialog::backdrop {
		background: rgb(0 0 0 / 0.45);
	}
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
	}
	h2 {
		margin: 0;
		font-size: 1.2rem;
	}
	h3 {
		margin: var(--space-3) 0 var(--space-1);
		font-size: 1rem;
	}
	p {
		margin: var(--space-2) 0;
	}
	ol,
	ul {
		margin: 0;
		padding-left: var(--space-4);
		display: grid;
		gap: var(--space-1);
	}
	ul.keys {
		list-style: none;
		padding-left: 0;
	}
	.kbds {
		display: inline-block;
		min-width: 9rem;
	}
	kbd {
		font-family: var(--font-mono);
		font-size: 0.85em;
		padding: 0 var(--space-1);
		border: 1px solid var(--color-border);
		border-radius: 3px;
		background: var(--color-bg);
	}
</style>
