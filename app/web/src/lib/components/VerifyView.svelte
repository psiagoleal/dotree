<!-- Caminho relativo: app/web/src/lib/components/VerifyView.svelte -->
<!-- Verification of the complete circuit (spec 04): runs in a worker, shows progress after
     `progressDelay` ms and can be cancelled (VER-04); disabled while incomplete (VER-05).
     The parent re-creates it when the circuit changes, so a result never outlives it. -->
<script lang="ts">
	import type { CompressionSession, Snapshot, VerificationProgress } from '#lib/core.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import {
		startVerification,
		type StartVerification,
		type VerificationRun
	} from '#lib/verification.ts';

	let {
		session,
		snap,
		start = startVerification,
		progressDelay = 500
	}: {
		session: CompressionSession;
		snap: Snapshot;
		/** Starts the worker; tests pass a fake. */
		start?: StartVerification;
		/** Progress stays hidden for runs shorter than this (VER-04). */
		progressDelay?: number;
	} = $props();

	type Status =
		| { kind: 'idle' }
		| { kind: 'running'; progress: VerificationProgress | null; visible: boolean }
		| { kind: 'done'; progress: VerificationProgress }
		| { kind: 'cancelled' }
		| { kind: 'error'; message: string };

	const id = $props.id();
	const netlist = $derived.by(() => {
		void snap;
		return session.netlist();
	});
	const fmt = new Intl.NumberFormat(getLocale());
	const width = $derived(snap.result?.length ?? 0);

	let status = $state<Status>({ kind: 'idle' });
	let run: VerificationRun | null = null;
	let timer: ReturnType<typeof setTimeout> | undefined;

	function verify() {
		if (netlist === null || status.kind === 'running') return;
		const r = start(netlist, (progress) => {
			if (run === r && status.kind === 'running') status.progress = progress;
		});
		run = r;
		status = { kind: 'running', progress: null, visible: false };
		timer = setTimeout(() => {
			if (run === r && status.kind === 'running') status.visible = true;
		}, progressDelay);
		const settle = (next: Status) => {
			if (run !== r) return;
			clearTimeout(timer);
			run = null;
			status = next;
		};
		r.done.then(
			(p) => settle(p ? { kind: 'done', progress: p } : { kind: 'cancelled' }),
			(e: unknown) => settle({ kind: 'error', message: e instanceof Error ? e.message : String(e) })
		);
	}

	// Leaving (a new circuit, a new exercise) stops the worker.
	$effect(() => () => {
		clearTimeout(timer);
		run?.cancel();
	});

	const binary = (value: string) => BigInt(value).toString(2).padStart(width, '0');
</script>

<section class="verify" aria-labelledby={`${id}-title`}>
	<h3 id={`${id}-title`}>{m.verify_title()}</h3>
	<div class="tools">
		<button
			type="button"
			onclick={verify}
			disabled={netlist === null || status.kind === 'running'}
			aria-describedby={netlist === null ? `${id}-incomplete` : undefined}
			>{m.action_verify()}</button
		>
		{#if status.kind === 'running' && status.visible}
			<button type="button" onclick={() => run?.cancel()}>{m.action_cancel()}</button>
		{/if}
	</div>
	{#if netlist === null}
		<p id={`${id}-incomplete`} class="note">{m.verify_incomplete()}</p>
	{/if}

	{#if status.kind === 'running' && status.visible}
		{@const p = status.progress}
		<div class="progress">
			<label for={`${id}-bar`}
				>{p
					? m.verify_running({ checked: fmt.format(p.checked), total: fmt.format(p.total) })
					: m.verify_started()}</label
			>
			<progress id={`${id}-bar`} max={p?.total} value={p?.checked}></progress>
		</div>
	{/if}

	<div role="status" data-testid="verify-result">
		{#if status.kind === 'done'}
			{@const p = status.progress}
			{#if p.counterexample === null}
				<p class="ok">
					{(p.coverage === 'exhaustive' ? m.verify_ok_exhaustive : m.verify_ok_sample)({
						count: fmt.format(p.checked)
					})}
				</p>
			{:else}
				{@const ce = p.counterexample}
				<p class="error">
					{m.verify_failed({ checked: fmt.format(p.checked), total: fmt.format(p.total) })}
				</p>
				<dl>
					<dt>{m.verify_inputs()}</dt>
					<dd>
						{#each ce.inputs as input, i (input.var)}{i > 0 ? ', ' : ''}<code
								>{input.var} = {input.value}</code
							>{/each}
					</dd>
					<dt>{m.verify_expected()}</dt>
					<dd><code>{ce.expected} = {binary(ce.expected)}₂</code></dd>
					<dt>{m.verify_obtained()}</dt>
					<dd><code>{ce.obtained} = {binary(ce.obtained)}₂</code></dd>
					<dt>{m.verify_differing()}</dt>
					<dd>
						{#each ce.differing as bit, i (bit)}{i > 0 ? ', ' : ''}<code>R({bit})</code>{/each}
					</dd>
				</dl>
			{/if}
		{:else if status.kind === 'cancelled'}
			<p class="note">{m.verify_cancelled()}</p>
		{:else if status.kind === 'error'}
			<p class="error">
				{status.message ? m.verify_error({ message: status.message }) : m.verify_error_load()}
			</p>
		{/if}
	</div>
</section>

<style>
	.verify {
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
	.ok,
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
	.tools {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
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
	.progress {
		display: grid;
		gap: var(--space-1);
		max-width: 28rem;
		font-size: 0.9rem;
		font-variant-numeric: tabular-nums;
	}
	progress {
		width: 100%;
		accent-color: var(--color-accent);
	}
	dl {
		display: grid;
		grid-template-columns: max-content 1fr;
		gap: var(--space-1) var(--space-3);
		margin: var(--space-2) 0 0;
		font-size: 0.9rem;
	}
	dt {
		font-weight: 600;
	}
	dd {
		margin: 0;
	}
	code {
		font-family: var(--font-mono);
	}
</style>
