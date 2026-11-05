<!-- Caminho relativo: app/web/src/routes/+page.svelte -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { m } from '#lib/paraglide/messages.js';
	import LanguageSwitch from '#lib/components/LanguageSwitch.svelte';
	import ExpressionForm from '#lib/components/ExpressionForm.svelte';
	import Compression from '#lib/components/Compression.svelte';
	import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
	import TermLegend from '#lib/components/TermLegend.svelte';
	import ViewToggle from '#lib/components/ViewToggle.svelte';
	import WidthChallenge from '#lib/components/WidthChallenge.svelte';
	import { loadCore, type BuildResult, type CompressionSession, type Core } from '#lib/core.ts';
	import { download } from '#lib/download.ts';
	import { errorMessage, fileErrorMessage, warningMessage } from '#lib/errors.ts';
	import { keyAction } from '#lib/keys.ts';
	import { autosaving, clearSaved, loadSaved, saveExercise } from '#lib/persist.ts';
	import { prewarmVerification } from '#lib/verification.ts';
	import { browserStorage, readView, writeView, type ViewMode } from '#lib/view.ts';

	const ISSUES_URL = 'https://github.com/psiagoleal/dotree/issues/new';
	const SPEC_URL = 'https://github.com/psiagoleal/dotree/blob/main/docs/specs/README.md';
	/** An exercise file is a few kilobytes; anything far larger is refused unread. */
	const MAX_FILE_BYTES = 1_000_000;

	type Status =
		{ kind: 'loading' } | { kind: 'ready'; core: Core } | { kind: 'error'; reason: string };

	let status = $state<Status>({ kind: 'loading' });
	let result = $state<BuildResult | null>(null);
	// The compression in progress, held by the core; freed when a new matrix replaces it.
	let session = $state.raw<CompressionSession | null>(null);
	// Dots or indexed variables, remembered across visits (VIS-01, VIS-03).
	let view = $state<ViewMode>(readView(browserStorage()));
	// W shows only after the width challenge is answered or revealed (EXP-09, GER-01).
	let widthRevealed = $state(false);
	// Mounted only once the core is ready, hence state.
	let discardDialog = $state<ConfirmDialog>();
	let form = $state<ExpressionForm>();
	let fileInput = $state<HTMLInputElement>();
	// Outcome of the last "open exercise" (SAL-03): a refusal leaves the exercise as it was.
	let fileStatus = $state<{ ok: boolean; text: string } | null>(null);
	// An exercise left in local storage by an earlier visit, waiting for the user's choice
	// (SAL-01), or one that could not be opened (SAL-05). Nothing overwrites it meanwhile.
	type Pending =
		| { kind: 'offer'; session: CompressionSession }
		| { kind: 'corrupt'; text: string; reason: string };
	let pending = $state.raw<Pending | null>(null);
	let pendingDialog = $state<ConfirmDialog>();
	// SAL-04: storage that fails is reported once; the app goes on without it.
	let storageFailed = $state(false);

	function save(text: string) {
		if (!saveExercise(browserStorage(), text)) storageFailed = true;
	}

	/** Makes `next` the exercise on screen, saving it now and after every change. */
	function adopt(next: CompressionSession | null) {
		session?.free();
		session = next && autosaving(next, save);
		if (next) save(next.exercise());
	}

	/** The saved exercise is dropped only after confirmation; true when it is gone. */
	async function settlePending(): Promise<boolean> {
		if (!pending) return true;
		if (!(await pendingDialog?.ask())) return false;
		discardPending();
		return true;
	}

	function discardPending() {
		if (pending?.kind === 'offer') pending.session.free();
		pending = null;
		clearSaved(browserStorage());
	}

	function checkSaved(core: Core) {
		const saved = loadSaved(browserStorage());
		if (saved.kind === 'unavailable') storageFailed = true;
		if (saved.kind !== 'found') return;
		const opened = core.restoreSession(saved.text);
		pending = opened.ok
			? { kind: 'offer', session: opened.session }
			: {
					kind: 'corrupt',
					text: saved.text,
					reason: fileErrorMessage(opened.error, undefined, true)
				};
	}

	function resume(core: Core) {
		if (pending?.kind !== 'offer') return;
		const next = pending.session;
		pending = null;
		show(core, next);
	}

	/** Puts a reopened exercise on screen: form, matrix, reveals and compression. */
	function show(core: Core, next: CompressionSession) {
		const expression = next.expression();
		const widths = next.widths();
		form?.load(expression, widths);
		result = core.buildMatrix(expression, widths);
		widthRevealed = next.revealed().width !== null;
		adopt(next);
	}

	async function build(core: Core, expression: string, widths: Record<string, number>) {
		if (!(await settlePending())) return;
		// EXP-04: a compression with any action in it is discarded only after confirmation.
		if (session?.snapshot().canUndo && !(await discardDialog?.ask())) return;
		fileStatus = null;
		result = core.buildMatrix(expression, widths);
		widthRevealed = false;
		adopt(result.ok ? core.openSession(expression, widths) : null);
	}

	function exportExercise() {
		if (session) download(m.file_name(), session.exercise(), 'application/json');
	}

	async function openFile(core: Core, file: File) {
		fileStatus = null;
		const refuse = (text: string) =>
			(fileStatus = { ok: false, text: m.file_import_failed({ file: file.name, reason: text }) });
		if (file.size > MAX_FILE_BYTES) return refuse(fileErrorMessage({ code: 'tooLarge' }));
		const opened = core.restoreSession(await file.text());
		if (!opened.ok) return refuse(fileErrorMessage(opened.error));
		const next = opened.session;
		const keep =
			!(await settlePending()) || (session?.snapshot().canUndo && !(await discardDialog?.ask()));
		if (keep) {
			next.free();
			return;
		}
		show(core, next);
		fileStatus = { ok: true, text: m.file_imported({ file: file.name }) };
	}

	function setView(mode: ViewMode) {
		view = mode;
		writeView(browserStorage(), mode);
	}

	function onkeydown(e: KeyboardEvent) {
		if (keyAction(e) !== 'toggleView') return;
		e.preventDefault();
		setView(view === 'dots' ? 'indexed' : 'dots');
	}

	onMount(() => {
		// Loading counts the verification worker too: after it, nothing is fetched (GER-05).
		Promise.all([loadCore(), prewarmVerification()])
			.then(([core]) => {
				status = { kind: 'ready', core };
				checkSaved(core);
			})
			.catch((err: unknown) => {
				// GER-09: a broken WASM load gets an explanation, not a broken interface.
				status = { kind: 'error', reason: err instanceof Error ? err.message : String(err) };
			});
	});
</script>

<svelte:window {onkeydown} />

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
		{#if pending?.kind === 'offer'}
			<div class="saved" role="status" data-testid="saved-offer">
				<p>{m.saved_offer({ expression: pending.session.expression() })}</p>
				<div class="buttons">
					<button type="button" class="primary" onclick={() => resume(core)}
						>{m.saved_resume()}</button
					>
					<button type="button" onclick={settlePending}>{m.saved_discard()}</button>
				</div>
			</div>
		{:else if pending?.kind === 'corrupt'}
			{@const text = pending.text}
			<div class="saved corrupt" role="alert" data-testid="saved-corrupt">
				<p>{m.saved_corrupt({ reason: pending.reason })}</p>
				<div class="buttons">
					<button
						type="button"
						onclick={() => download(m.saved_file_name(), text, 'application/json')}
						>{m.saved_download()}</button
					>
					<button type="button" onclick={settlePending}>{m.saved_discard()}</button>
				</div>
			</div>
		{/if}
		{#if storageFailed}
			<p class="note" role="status" data-testid="storage-warning">{m.storage_unavailable()}</p>
		{/if}
		<ConfirmDialog
			bind:this={pendingDialog}
			title={m.saved_confirm_title()}
			body={m.saved_confirm_body()}
			cancelLabel={m.saved_confirm_cancel()}
			confirmLabel={m.saved_confirm_ok()}
		/>
		<ExpressionForm bind:this={form} onbuild={(e, w) => build(core, e, w)} />
		<div class="file">
			<button type="button" onclick={exportExercise} disabled={!session}>{m.file_export()}</button>
			<button type="button" onclick={() => fileInput?.click()}>{m.file_import()}</button>
			<input
				bind:this={fileInput}
				type="file"
				accept=".json,application/json"
				hidden
				data-testid="file-input"
				onchange={(e) => {
					const file = e.currentTarget.files?.[0];
					e.currentTarget.value = '';
					if (file) void openFile(core, file);
				}}
			/>
		</div>
		<div role="status" data-testid="file-status" class:error-text={fileStatus && !fileStatus.ok}>
			{#if fileStatus}<p>{fileStatus.text}</p>{/if}
		</div>
		<ConfirmDialog
			bind:this={discardDialog}
			title={m.discard_confirm_title()}
			body={m.discard_confirm_body()}
			cancelLabel={m.discard_confirm_cancel()}
			confirmLabel={m.discard_confirm_ok()}
		/>

		{#if result && !result.ok}
			<p role="alert" class="error" data-testid="build-error">{errorMessage(result.error)}</p>
		{:else if result && result.ok}
			{@const mx = result.matrix}
			<section class="result" aria-labelledby="matrix-title">
				<div class="result-head">
					<h2 id="matrix-title">{m.matrix_title()}</h2>
					<ViewToggle mode={view} onchange={setView} />
				</div>
				{#each mx.warnings as w, i (i)}
					<p class="warning" role="status">{warningMessage(w)}</p>
				{/each}
				<p class="summary" data-testid="matrix-summary">
					{m.matrix_summary({ dots: mx.dots.length, ands: mx.andGates })}
				</p>
				<TermLegend terms={mx.terms} />
				{#key mx}
					<WidthChallenge
						matrix={mx}
						initial={session?.revealed().width ?? null}
						onreveal={(answer) => {
							session?.revealWidth(answer);
							widthRevealed = true;
						}}
						onpath={() => session?.revealWidthPath()}
					/>
				{/key}
				{#if session}
					{#key session}
						<Compression
							{session}
							{view}
							outputWidth={mx.outputWidth}
							vMax={mx.vMax}
							{widthRevealed}
						/>
					{/key}
				{/if}
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
	.file {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		margin-top: var(--space-2);
	}
	.file button {
		font: inherit;
		padding: var(--space-1) var(--space-3);
		border-radius: var(--radius);
		border: 1px solid var(--color-accent);
		background: transparent;
		color: var(--color-accent);
		cursor: pointer;
	}
	.file button:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}
	[data-testid='file-status'] p {
		margin: var(--space-2) 0 0;
		font-size: 0.9rem;
	}
	.error-text {
		color: var(--color-danger);
	}
	.saved {
		display: grid;
		gap: var(--space-2);
		margin-bottom: var(--space-3);
		padding: var(--space-3);
		border: 1px solid var(--color-accent);
		border-radius: var(--radius);
		background: var(--color-surface);
	}
	.saved.corrupt {
		border-color: var(--color-danger);
	}
	.saved p {
		margin: 0;
	}
	.saved .buttons {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
	.saved button {
		font: inherit;
		padding: var(--space-1) var(--space-3);
		border-radius: var(--radius);
		border: 1px solid var(--color-accent);
		background: transparent;
		color: var(--color-accent);
		cursor: pointer;
	}
	.saved button.primary {
		background: var(--color-accent);
		color: var(--color-bg);
	}
	p.note {
		margin: 0 0 var(--space-3);
		color: var(--color-muted);
		font-size: 0.9rem;
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
	.result-head {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
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
