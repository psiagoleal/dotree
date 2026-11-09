// Caminho relativo: app/web/e2e/verify.e2e.ts
// MT-14b: verification in a worker (spec 04) and the VHDL files (spec 06).
import { execFileSync } from 'node:child_process';
import { expect, test, type Page } from '@playwright/test';
import { r2ByMouse } from './r2';

test.use({ locale: 'pt-BR', permissions: ['clipboard-read', 'clipboard-write'] });

async function r2Complete(page: Page) {
	await page.goto('./');
	await r2ByMouse(page);
	await page.getByRole('button', { name: 'Fechar com CPA' }).click();
}

test('R2: 16 384 cases, exhaustive, 0 errors (VER-01)', async ({ page }) => {
	await r2Complete(page);
	await page.getByRole('button', { name: 'Verificar circuito' }).click();
	await expect(page.getByTestId('verify-result')).toHaveText('16.384 casos, exaustivo, 0 erros.');
	// A result belongs to the circuit it checked: undoing the CPA drops it.
	await page.getByRole('button', { name: 'Desfazer' }).click();
	await expect(page.getByTestId('verify-result')).toHaveText('');
	await expect(page.getByRole('button', { name: 'Verificar circuito' })).toBeDisabled();
});

test('a long verification shows progress, keeps the page alive and can be cancelled (VER-04)', async ({
	page
}) => {
	// 24 input bits: 2^24 cases, the longest run of spec 04, closed straight with the CPA.
	const problems: string[] = [];
	page.on('pageerror', (e) => problems.push(`pageerror: ${e.message}`));
	page.on('console', (msg) => {
		if (msg.type() === 'error') problems.push(`console: ${msg.text()}`);
	});
	await page.goto('./');
	await page.getByLabel('Expressão').fill('A + B');
	for (const v of ['A', 'B']) await page.locator(`input[name="width-${v}"]`).fill('12');
	await page.getByRole('button', { name: 'Montar a matriz' }).click();
	await page.getByRole('button', { name: 'Fechar com CPA' }).click();

	await page.getByRole('button', { name: 'Verificar circuito' }).click();
	const bar = page.getByRole('progressbar');
	// The bar shows only after 500 ms of running. If it never does, say what the page
	// showed instead: a run that ended or failed early, or errors from the page.
	try {
		await expect(bar).toBeVisible();
	} catch (e) {
		const result = await page.getByTestId('verify-result').textContent();
		throw new Error(
			`${e}\nverify-result: ${JSON.stringify(result)}\n${problems.join('\n') || 'no page errors'}`,
			{ cause: e }
		);
	}
	await expect(page.getByText(/^Verificando: [\d.]+ de 16\.777\.216 casos$/)).toBeVisible();
	// The bar keeps moving: the page handles the worker's messages while it runs.
	const first = Number(await bar.getAttribute('value'));
	await expect.poll(async () => Number(await bar.getAttribute('value'))).toBeGreaterThan(first);

	await page.getByRole('button', { name: 'Cancelar' }).click();
	await expect(page.getByTestId('verify-result')).toHaveText(
		'Verificação cancelada; nenhum resultado foi registrado.'
	);
	await expect(bar).toHaveCount(0);
	await expect(page.getByText(/casos, exaustivo/)).toHaveCount(0);
	await expect(page.getByRole('button', { name: 'Verificar circuito' })).toBeEnabled();
});

test('R2: copy each VHDL file and download them all as a .zip (VHD-05)', async ({
	page
}, testInfo) => {
	await r2Complete(page);
	const names = [
		'full_adder.vhd',
		'half_adder.vhd',
		'dotree_circuito.vhd',
		'dotree_circuito_comportamental.vhd',
		'tb_dotree_circuito.vhd',
		// spec 10: R2 fits the DE2 board
		'dotree_circuito_de2.vhd',
		'tb_dotree_circuito_de2.vhd',
		'dotree_circuito.qpf',
		'dotree_circuito.qsf'
	];
	const contents: Record<string, string> = {};
	for (const name of names) {
		contents[name] = (await page.locator(`pre[data-file="${name}"]`).textContent()) ?? '';
		await page.getByRole('button', { name: `Copiar ${name}` }).click();
		await expect(page.getByText(`${name} copiado.`)).toBeVisible();
		expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(contents[name]);
	}
	expect(contents['dotree_circuito.vhd']).toContain('entity dotree_circuito is');
	// VHD-10: the behavioral twin, and the testbench that checks both.
	expect(contents['dotree_circuito_comportamental.vhd']).toContain(
		'entity dotree_circuito_comportamental is'
	);
	expect(contents['tb_dotree_circuito.vhd']).toContain(
		'entity work.dotree_circuito_comportamental'
	);
	expect(contents['dotree_circuito.qsf']).toContain(
		'set_global_assignment -name DEVICE EP2C35F672C6'
	);

	const downloading = page.waitForEvent('download');
	await page.getByRole('button', { name: 'Baixar todos (.zip)' }).click();
	const zip = await downloading;
	expect(zip.suggestedFilename()).toBe('dotree_circuito.zip');
	const path = testInfo.outputPath('vhdl.zip');
	await zip.saveAs(path);
	// Read back by an independent implementation: Python's zipfile, checking every CRC.
	const read = execFileSync(
		'python3',
		[
			'-I',
			'-c',
			'import json, sys, zipfile\n' +
				'z = zipfile.ZipFile(sys.argv[1])\n' +
				'assert z.testzip() is None\n' +
				'print(json.dumps({n: z.read(n).decode() for n in z.namelist()}))',
			path
		],
		{ encoding: 'utf8' }
	);
	expect(JSON.parse(read)).toEqual(contents);
});

test('VHDL and verification wait for a complete circuit (VER-05, VHD-07)', async ({ page }) => {
	await page.goto('./');
	await r2ByMouse(page);
	await expect(page.getByRole('button', { name: 'Verificar circuito' })).toBeDisabled();
	await expect(page.getByText('A verificação exige o circuito completo.')).toBeVisible();
	await expect(page.getByRole('button', { name: 'Baixar todos (.zip)' })).toBeDisabled();
	await expect(page.getByText('O VHDL exige o circuito completo.')).toBeVisible();
});
