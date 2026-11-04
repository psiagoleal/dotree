// Caminho relativo: app/web/src/lib/components/VerifyView.svelte.spec.ts
// Verification panel (spec 04): the real worker on R2, and a fake one to control timing.
import { beforeAll, describe, expect, it } from 'vitest';
import { page } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import VerifyView from './VerifyView.svelte';
import { loadCore, type Core, type VerificationProgress } from '#lib/core.ts';
import type { StartVerification } from '#lib/verification.ts';
import { r2, r2Levels } from '#lib/testing/r2.ts';

let core: Core;

beforeAll(async () => {
	core = await loadCore();
});

const verifyButton = () => page.getByRole('button', { name: 'Verificar circuito' });
const cancelButton = () => page.getByRole('button', { name: 'Cancelar' });
const result = () => page.getByTestId('verify-result');

/** A fake run the test drives: `push` sends progress, `finish` ends it. */
function controlled() {
	let onprogress: (p: VerificationProgress) => void = () => {};
	let resolve: (p: VerificationProgress | null) => void = () => {};
	let cancelled = false;
	const start: StartVerification = (_netlist, cb) => {
		onprogress = cb;
		return {
			done: new Promise((r) => (resolve = r)),
			cancel: () => {
				cancelled = true;
				resolve(null);
			}
		};
	};
	return {
		start,
		push: (p: VerificationProgress) => onprogress(p),
		finish: (p: VerificationProgress) => resolve(p),
		cancelled: () => cancelled
	};
}

const at = (checked: number, extra: Partial<VerificationProgress> = {}): VerificationProgress => ({
	checked,
	total: 16384,
	coverage: 'exhaustive',
	done: checked === 16384,
	counterexample: null,
	...extra
});

describe('VerifyView (VER-01, VER-03 to VER-05)', () => {
	it('is disabled while the circuit is incomplete, saying why (VER-05)', async () => {
		const s = r2Levels(core, 3);
		await render(VerifyView, { session: s, snap: s.snapshot() });
		await expect.element(verifyButton()).toBeDisabled();
		await expect.element(page.getByText('A verificação exige o circuito completo.')).toBeVisible();
	});

	it('R2 in the real worker: 16 384 cases, exhaustive, 0 errors (VER-01)', async () => {
		const s = r2(core);
		await render(VerifyView, { session: s, snap: s.snapshot() });
		await verifyButton().click();
		await expect.element(result()).toHaveTextContent('16.384 casos, exaustivo, 0 erros.');
	});

	it('shows progress only after the delay, and cancelling leaves no result (VER-04)', async () => {
		const c = controlled();
		const s = r2(core);
		await render(VerifyView, {
			session: s,
			snap: s.snapshot(),
			start: c.start,
			progressDelay: 200
		});
		await verifyButton().click();
		await expect.element(verifyButton()).toBeDisabled();
		c.push(at(1024));
		expect(document.querySelector('progress')).toBeNull();
		await expect.element(page.getByText('Verificando: 1.024 de 16.384 casos')).toBeVisible();
		await expect.element(page.getByRole('progressbar')).toHaveAttribute('value', '1024');
		await cancelButton().click();
		expect(c.cancelled()).toBe(true);
		await expect
			.element(result())
			.toHaveTextContent('Verificação cancelada; nenhum resultado foi registrado.');
		expect(document.querySelector('progress')).toBeNull();
		await expect.element(cancelButton()).not.toBeInTheDocument();
		await expect.element(verifyButton()).toBeEnabled();
	});

	it('a quick run never shows progress', async () => {
		const c = controlled();
		const s = r2(core);
		await render(VerifyView, {
			session: s,
			snap: s.snapshot(),
			start: c.start,
			progressDelay: 200
		});
		await verifyButton().click();
		c.finish(at(16384, { coverage: 'sample' }));
		await expect.element(result()).toHaveTextContent('16.384 casos, por amostra, 0 erros.');
		await new Promise((r) => setTimeout(r, 300));
		expect(document.querySelector('progress')).toBeNull();
	});

	it('shows the first counterexample: inputs, expected, obtained, differing bits (VER-03)', async () => {
		const c = controlled();
		const s = r2(core);
		await render(VerifyView, { session: s, snap: s.snapshot(), start: c.start });
		await verifyButton().click();
		c.finish(
			at(37, {
				done: true,
				counterexample: {
					inputs: [
						{ var: 'A', value: '5' },
						{ var: 'B', value: '0' }
					],
					expected: '24',
					obtained: '16',
					differing: [3]
				}
			})
		);
		const r = result();
		await expect.element(r).toHaveTextContent(/^Erro no caso 37 de 16\.384/);
		await expect.element(r).toHaveTextContent('A = 5, B = 0');
		await expect.element(r).toHaveTextContent('24 = 0011000₂');
		await expect.element(r).toHaveTextContent('16 = 0010000₂');
		await expect.element(r).toHaveTextContent('R(3)');
	});
});
