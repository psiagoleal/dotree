<!-- Caminho relativo: app/web/src/lib/components/VhdlView.svelte -->
<!-- VHDL of the complete circuit (spec 06): entity name checked by the core (VHD-08), copy
     of each file and a .zip with all of them (VHD-05); disabled, with the reason, while the
     circuit is incomplete (VHD-07). Comments follow the interface language (VHD-06). Says
     whether the DE2 board top and Quartus project are included, or why not (spec 10). -->
<script lang="ts">
	import type { CompressionSession, Snapshot, VhdlFile } from '#lib/core.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import { download } from '#lib/download.ts';
	import { zip } from '#lib/zip.ts';

	let { session, snap }: { session: CompressionSession; snap: Snapshot } = $props();

	const id = $props.id();
	let name = $state('dotree_circuito');
	let copyStatus = $state('');

	const result = $derived.by(() => {
		void snap;
		return session.vhdl(name, getLocale() === 'en' ? 'en' : 'pt-br');
	});
	const incomplete = $derived(!result.ok && result.error === 'notComplete');
	const invalid = $derived(!result.ok && result.error === 'invalidName');
	const files = $derived(result.ok ? result.files : []);
	const boardNote = $derived.by(() => {
		if (!result.ok) return '';
		const b = result.board;
		if (b.fits) return m.vhdl_board_ok({ file: `${name}_de2.vhd` });
		return b.reason === 'tooManyInputs'
			? m.vhdl_board_too_many_inputs({ max: b.max, bits: b.bits })
			: m.vhdl_board_too_wide({ max: b.max, width: b.width });
	});

	function copy(f: VhdlFile) {
		navigator.clipboard.writeText(f.contents).then(
			() => (copyStatus = m.vhdl_copied({ file: f.name })),
			() => (copyStatus = m.vhdl_copy_failed({ file: f.name }))
		);
	}

	function downloadZip() {
		if (!result.ok) return;
		download(`${name}.zip`, zip(result.files), 'application/zip');
	}
</script>

<section class="vhdl" aria-labelledby={`${id}-title`}>
	<h3 id={`${id}-title`}>{m.vhdl_title()}</h3>
	{#if incomplete}
		<div class="tools">
			<button type="button" disabled aria-describedby={`${id}-incomplete`}>{m.vhdl_zip()}</button>
		</div>
		<p id={`${id}-incomplete`} class="note">{m.vhdl_incomplete()}</p>
	{:else}
		<div class="tools">
			<label for={`${id}-name`}>{m.vhdl_name()}</label>
			<input
				id={`${id}-name`}
				type="text"
				bind:value={name}
				spellcheck="false"
				autocomplete="off"
				aria-invalid={invalid}
				aria-describedby={invalid ? `${id}-invalid` : undefined}
			/>
			<button type="button" onclick={downloadZip} disabled={invalid}>{m.vhdl_zip()}</button>
		</div>
		{#if invalid}
			<p id={`${id}-invalid`} class="error">{m.vhdl_name_invalid()}</p>
		{:else}
			<p class="note">{boardNote}</p>
		{/if}
		<ul class="files">
			{#each files as f (f.name)}
				<li>
					<div class="file-head">
						<code>{f.name}</code>
						<button type="button" onclick={() => copy(f)}>{m.vhdl_copy({ file: f.name })}</button>
					</div>
					<details>
						<summary>{m.vhdl_show()}</summary>
						<pre data-file={f.name}><code>{f.contents}</code></pre>
					</details>
				</li>
			{/each}
		</ul>
		<p class="note" role="status">{copyStatus}</p>
	{/if}
</section>

<style>
	.vhdl {
		display: grid;
		gap: var(--space-2);
		margin-top: var(--space-3);
		padding-top: var(--space-3);
		border-top: 1px solid var(--color-border);
	}
	h3 {
		margin: 0;
		font-size: 1rem;
	}
	.note,
	.error {
		margin: 0;
		font-size: 0.9rem;
	}
	.note {
		color: var(--color-muted);
	}
	.error {
		color: var(--color-danger);
	}
	.tools,
	.file-head {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
	}
	input {
		font: inherit;
		font-family: var(--font-mono);
		padding: var(--space-1) var(--space-2);
		border: 1px solid var(--color-border);
		border-radius: var(--radius);
		background: var(--color-surface);
		color: var(--color-text);
	}
	input[aria-invalid='true'] {
		border-color: var(--color-danger);
	}
	button {
		font: inherit;
		padding: var(--space-1) var(--space-3);
		border-radius: var(--radius);
		border: 1px solid var(--color-accent);
		background: transparent;
		color: var(--color-accent);
		cursor: pointer;
	}
	button:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}
	.files {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: var(--space-2);
	}
	summary {
		cursor: pointer;
		font-size: 0.9rem;
	}
	pre {
		max-height: 24rem;
		overflow: auto;
		margin: var(--space-1) 0 0;
		padding: var(--space-2);
		border: 1px solid var(--color-border);
		border-radius: var(--radius);
		background: var(--color-surface);
		font-size: 0.8rem;
	}
	code {
		font-family: var(--font-mono);
	}
</style>
