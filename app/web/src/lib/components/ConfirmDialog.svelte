<!-- Caminho relativo: app/web/src/lib/components/ConfirmDialog.svelte -->
<!-- The question asked before any answer is shown (AJU-02, D-5): the focus starts on "keep
     trying", confirming is the second option, and Escape counts as keep trying. Callers do
     `if (await dialog.ask()) reveal()`; every request asks again. -->
<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';

	let { title, body }: { title: string; body: string } = $props();

	const id = $props.id();
	let dialog: HTMLDialogElement;
	let keepTrying: HTMLButtonElement;
	let open = $state(false);
	let resolve: ((confirmed: boolean) => void) | null = null;

	export function ask(): Promise<boolean> {
		finish(false); // a request still pending counts as keep trying
		open = true;
		dialog.showModal();
		keepTrying.focus();
		return new Promise((r) => (resolve = r));
	}

	function finish(confirmed: boolean) {
		const r = resolve;
		resolve = null;
		open = false;
		if (dialog.open) dialog.close();
		r?.(confirmed);
	}
</script>

<dialog
	bind:this={dialog}
	aria-labelledby={`${id}-title`}
	aria-describedby={`${id}-body`}
	onclose={() => finish(false)}
>
	{#if open}
		<h2 id={`${id}-title`}>{title}</h2>
		<p id={`${id}-body`}>{body}</p>
	{/if}
	<div class="actions">
		<button bind:this={keepTrying} type="button" class="primary" onclick={() => finish(false)}>
			{m.confirm_keep_trying()}
		</button>
		<button type="button" class="secondary" onclick={() => finish(true)}>
			{m.confirm_show()}
		</button>
	</div>
</dialog>

<style>
	dialog {
		max-width: min(28rem, calc(100vw - 2rem));
		padding: var(--space-4);
		border: 1px solid var(--color-border);
		border-radius: var(--radius);
		background: var(--color-surface);
		color: var(--color-text);
	}
	dialog::backdrop {
		background: rgb(0 0 0 / 0.45);
	}
	h2 {
		margin: 0 0 var(--space-2);
		font-size: 1.15rem;
	}
	p {
		margin: 0 0 var(--space-3);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
	button {
		font: inherit;
		padding: var(--space-1) var(--space-3);
		border-radius: var(--radius);
		border: 1px solid var(--color-accent);
		cursor: pointer;
	}
	.primary {
		background: var(--color-accent);
		color: var(--color-bg);
	}
	.secondary {
		background: transparent;
		color: var(--color-accent);
	}
</style>
