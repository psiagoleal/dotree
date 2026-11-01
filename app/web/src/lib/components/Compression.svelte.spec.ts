// Caminho relativo: app/web/src/lib/components/Compression.svelte.spec.ts
// Drives the real core (WASM) through the compression panel with R1 (spec 00).
import { beforeAll, describe, expect, it } from 'vitest';
import { page, userEvent } from 'vitest/browser';
import { render } from 'vitest-browser-svelte';
import Compression from './Compression.svelte';
import { loadCore, type CompressionSession, type Core } from '#lib/core.ts';
import { EXAMPLES } from '#lib/expression.ts';

let core: Core;
const r1 = EXAMPLES.find((e) => e.id === 'r1')!;

beforeAll(async () => {
	core = await loadCore();
});

async function open(view: 'dots' | 'indexed' = 'dots') {
	const session = core.openSession(r1.expression, r1.widths);
	return render(Compression, { session, view, ...R1_WIDTH });
}

// R1: W = 7, v_max = 79 (spec 00); hidden unless a test reveals it.
const R1_WIDTH = { outputWidth: 7, vMax: '79', widthRevealed: false };

/** Forms a cell straight on the session, by signal, before the component reads it. */
function formOn(s: CompressionSession, column: number, signals: string[]) {
	const points = s.snapshot().current.points;
	const ids = signals.map((sig) => points.find((p) => p.column === column && p.signal === sig)!.id);
	expect(s.formCell(ids).ok).toBe(true);
}

/** Levels 0 to 2 of the R2 tree (spec 00): level 3 is ready for the CPA. */
function r2BeforeTheCpa(): CompressionSession {
	const s = core.openSession(r1.expression, r1.widths);
	const levels: [number, string[]][][] = [
		[
			[1, ['a0_b1', 'a1_b0', 'c0_d1']],
			[1, ['c1_d0', 'e1', 'e0']],
			[2, ['a0_b2', 'a1_b1']]
		],
		[
			[1, ['f0', 's1', 's2']],
			[2, ['a2_b0', 'c1_d1', 'e1']],
			[2, ['s3', 'k1', 'k2']],
			[3, ['a1_b2', 'a2_b1', 'k3']]
		],
		[
			[0, ['a0_b0', 'c0_d0', 'e0']],
			[2, ['s5', 'k4', 's6']],
			[3, ['k5', 's7', 'k6']],
			[4, ['a2_b2', 'k7']]
		]
	];
	for (const cells of levels) {
		for (const [column, signals] of cells) formOn(s, column, signals);
		expect(s.concludeLevel().ok).toBe(true);
	}
	return s;
}

/** The point with this signal in column `column` of the current level. */
const point = (signal: string, column: number) =>
	page
		.getByTestId('level-current')
		.getByRole('group', { name: new RegExp(`2\\^${column},`) })
		.getByRole('button', { name: signal, exact: true });

describe('Compression (CMP-01, CMP-02, CMP-05, CMP-13 to CMP-15, GER-02)', () => {
	it('selection survives switching the view (VIS-02)', async () => {
		const { rerender } = await open();
		await point('a0_b2', 2).click();
		await point('a1_b1', 2).click();
		await expect.element(page.getByText('Seleção: 2')).toBeVisible();
		await rerender({ view: 'indexed' });
		await expect.element(point('a0_b2', 2)).toHaveAttribute('aria-pressed', 'true');
		await expect.element(point('a0_b2', 2)).toHaveTextContent('a0b2');
		await expect.element(point('a1_b1', 2)).toHaveAttribute('aria-pressed', 'true');
	});

	it('forming a cell boxes its points and counts it at once', async () => {
		await open();
		await point('a0_b2', 2).click();
		await point('a1_b1', 2).click();
		await page.getByRole('button', { name: 'Formar célula' }).click();
		const box = page.getByTestId('level-current').getByRole('group', { name: 'HA pendente' });
		await expect.element(box).toBeVisible();
		expect(box.element().querySelectorAll('.dot')).toHaveLength(2);
		await expect.element(page.getByTestId('metrics')).toHaveTextContent(/HA\s*1/);
		await expect.element(page.getByTestId('metrics')).toHaveTextContent(/Área\s*0,5/);
		await expect.element(page.getByTestId('metrics')).toHaveTextContent(/Atraso\s*0,5/);
		await expect.element(page.getByText('Seleção: 0')).toBeVisible();
	});

	it('a refusal states the rule and keeps the selection (GER-02, CMP-07)', async () => {
		await open();
		await point('a0_b0', 0).click();
		await page.getByRole('button', { name: 'Formar célula' }).click();
		await expect
			.element(page.getByRole('alert'))
			.toHaveTextContent('Um FA usa 3 pontos; um HA usa 2. A seleção tem 1.');
		await expect.element(point('a0_b0', 0)).toHaveAttribute('aria-pressed', 'true');
		expect(
			page
				.getByTestId('level-current')
				.getByRole('group', { name: /pendente/ })
				.elements()
		).toHaveLength(0);
	});

	it('concluding a level adds a panel; the cell gets its number (CMP-05)', async () => {
		await open();
		await page.getByRole('button', { name: 'Concluir nível' }).click();
		await expect
			.element(page.getByRole('alert'))
			.toHaveTextContent('Um nível precisa de pelo menos uma célula.');

		await point('a0_b1', 1).click();
		await point('a1_b0', 1).click();
		await point('c0_d1', 1).click();
		await page.getByRole('button', { name: 'Formar célula' }).click();
		await page.getByRole('button', { name: 'Concluir nível' }).click();

		await expect
			.element(page.getByTestId('level-0').getByRole('group', { name: 'FA 1' }))
			.toBeVisible();
		await expect.element(page.getByRole('heading', { name: 'Nível 1 (atual)' })).toBeVisible();
		await expect.element(point('s1', 1)).toBeVisible();
		await expect.element(point('k1', 2)).toBeVisible();
		await expect
			.element(page.getByTestId('level-current').getByRole('group', { name: /2\^1,/ }))
			.toHaveAccessibleName('Coluna de peso 2^1, 5 pontos');
	});

	it('keyboard: F forms, Ctrl+Z undoes, Ctrl+Y redoes, Esc clears (GER-07, CMP-03)', async () => {
		await open();
		const undo = page.getByRole('button', { name: 'Desfazer' });
		await expect.element(undo).toBeDisabled();
		await point('a0_b2', 2).click();
		await point('a1_b1', 2).click();
		await userEvent.keyboard('f');
		const ha = page.getByTestId('level-current').getByRole('group', { name: 'HA pendente' });
		await expect.element(ha).toBeVisible();
		await expect.element(undo).toBeEnabled();

		await userEvent.keyboard('{Control>}z{/Control}');
		await expect.element(ha).not.toBeInTheDocument();
		await userEvent.keyboard('{Control>}y{/Control}');
		await expect.element(ha).toBeVisible();

		await point('a0_b0', 0).click();
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByText('Seleção: 0')).toBeVisible();
	});
});

describe('Compression up to the result (CMP-04, CMP-09 to CMP-12, CMP-15)', () => {
	const cpa = () => page.getByRole('button', { name: 'Fechar com CPA' });

	it('the CPA button stays disabled and says which condition is missing (CMP-10)', async () => {
		await open();
		await expect.element(cpa()).toBeDisabled();
		await expect
			.element(cpa())
			.toHaveAccessibleDescription(
				'Fechar com CPA: A coluna 2^0 tem 5 pontos; o CPA aceita no máximo 3.'
			);
	});

	it('R2 closed with the CPA: 5 panels, 17 boxes, R0 to R6 and the critical path (CMP-05, CMP-12, CMP-15)', async () => {
		const { container } = await render(Compression, {
			session: r2BeforeTheCpa(),
			view: 'dots',
			...R1_WIDTH
		});
		await expect.element(cpa()).toBeEnabled();
		await cpa().click();

		await expect.element(page.getByRole('heading', { name: 'Nível 3: CPA' })).toBeVisible();
		await expect.element(page.getByRole('heading', { name: 'Resultado' })).toBeVisible();
		expect(container.querySelectorAll('section.panel')).toHaveLength(5);
		expect(container.querySelectorAll('.cell')).toHaveLength(17);
		await expect
			.element(page.getByTestId('complete'))
			.toHaveTextContent('Circuito completo: o resultado tem 7 bits, de R0 a R6.');
		for (let j = 0; j <= 6; j++) {
			await expect
				.element(
					page
						.getByTestId('result')
						.getByRole('group', { name: `Coluna de peso 2^${j}, bit R${j}` })
				)
				.toBeVisible();
		}
		await expect
			.element(page.getByRole('button', { name: 'Formar célula' }))
			.not.toBeInTheDocument();
		await expect.element(page.getByTestId('metrics')).toHaveTextContent(/FA\s*13/);

		expect(container.querySelectorAll('.cell.critical')).toHaveLength(0);
		await page.getByRole('button', { name: 'Destacar caminho crítico' }).click();
		const critical = [...container.querySelectorAll('.cell.critical')]
			.map((c) => c.getAttribute('aria-label')!)
			.sort((a, b) => parseInt(a.slice(3)) - parseInt(b.slice(3)));
		expect(critical).toEqual(
			[1, 2, 4, 6, 8, 9, 12, 13, 14, 15, 16, 17].map((n) =>
				expect.stringMatching(new RegExp(`^[FH]A ${n}, no caminho crítico$`))
			)
		);
	});

	it('going back asks first when cells would be discarded; cancelling changes nothing (CMP-04)', async () => {
		await render(Compression, { session: r2BeforeTheCpa(), view: 'dots', ...R1_WIDTH });
		const back = page.getByTestId('level-1').getByRole('button', { name: 'Voltar a este nível' });

		await back.click();
		const stay = page.getByRole('button', { name: 'Ficar onde estou' });
		await expect.element(stay).toHaveFocus();
		await stay.click();
		await expect.element(page.getByRole('heading', { name: 'Nível 3 (atual)' })).toBeVisible();

		await back.click();
		await page.getByRole('button', { name: 'Voltar', exact: true }).click();
		await expect.element(page.getByRole('heading', { name: 'Nível 1 (atual)' })).toBeVisible();
		expect(
			page
				.getByTestId('level-current')
				.getByRole('group', { name: /pendente/ })
				.elements()
		).toHaveLength(4);
	});

	it('going back without discarding anything does not ask (CMP-04)', async () => {
		const s = core.openSession(r1.expression, r1.widths);
		formOn(s, 2, ['a0_b2', 'a1_b1']);
		s.concludeLevel();
		await render(Compression, { session: s, view: 'dots', ...R1_WIDTH });
		await page.getByTestId('level-0').getByRole('button', { name: 'Voltar a este nível' }).click();
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
		await expect.element(page.getByRole('heading', { name: 'Nível 0 (atual)' })).toBeVisible();
		await expect
			.element(page.getByTestId('level-current').getByRole('group', { name: 'HA pendente' }))
			.toBeVisible();
	});

	/** A + B + C of 1 bit each (W = 2): the last HA's carry would land in 2^2 (CMP-09). */
	function droppedCarry(): CompressionSession {
		const s = core.openSession('A + B + C', { A: 1, B: 1, C: 1 });
		formOn(s, 0, ['a0', 'b0']);
		s.concludeLevel();
		formOn(s, 0, ['c0', 's1']);
		s.concludeLevel();
		formOn(s, 1, ['k1', 'k2']);
		s.concludeLevel();
		return s;
	}

	it('a dropped carry is marked; its explanation reveals W only after confirmation (CMP-09)', async () => {
		await render(Compression, {
			session: droppedCarry(),
			view: 'dots',
			outputWidth: 2,
			vMax: '3',
			widthRevealed: false
		});
		await expect
			.element(page.getByRole('group', { name: 'HA 3, vai-um sempre 0' }))
			.toHaveTextContent('k=0');
		await expect
			.element(page.getByText('O vai-um de HA 3 vale sempre 0 e não vira ponto.'))
			.toBeVisible();
		await expect.element(page.getByTestId('dropped-explanation')).not.toBeInTheDocument();

		await page.getByRole('button', { name: 'Por quê?' }).click();
		await page.getByRole('button', { name: 'Vou tentar mais um pouco' }).click();
		await expect.element(page.getByTestId('dropped-explanation')).not.toBeInTheDocument();

		await page.getByRole('button', { name: 'Por quê?' }).click();
		await page.getByRole('button', { name: 'Mostrar' }).click();
		await expect.element(page.getByTestId('dropped-explanation')).toHaveTextContent('W = 2');
		await expect.element(page.getByTestId('complete')).toBeVisible();
	});

	it('once W is revealed, the explanation needs no confirmation (CMP-09)', async () => {
		await render(Compression, {
			session: droppedCarry(),
			view: 'dots',
			outputWidth: 2,
			vMax: '3',
			widthRevealed: true
		});
		await page.getByRole('button', { name: 'Por quê?' }).click();
		await expect.element(page.getByTestId('dropped-explanation')).toBeVisible();
	});
});

describe('Compression with help (AJU-04, AJU-07)', () => {
	const show = () => page.getByRole('dialog').getByRole('button', { name: 'Mostrar' });

	it('the next move is highlighted in the matrix; nothing is formed until asked', async () => {
		const { container } = await open();
		await page.getByRole('button', { name: 'Próxima jogada' }).click();
		await show().click();
		const suggested = () =>
			[...container.querySelectorAll('.dot.suggested')].map((d) => d.getAttribute('data-signal'));
		expect(suggested()).toEqual(['a0_b1', 'a1_b0']);
		const pending = page.getByTestId('level-current').getByRole('group', { name: 'HA pendente' });
		await expect.element(pending).not.toBeInTheDocument();

		await page.getByRole('button', { name: 'Formar esta célula' }).click();
		await expect.element(pending).toBeVisible();
		expect(suggested()).toEqual([]);
		await expect.element(page.getByTestId('next-move')).not.toBeInTheDocument();
	});

	it('an answer about the current level disappears after the next action', async () => {
		await open();
		await page.getByRole('button', { name: 'Dica' }).click();
		await show().click();
		await expect.element(page.getByTestId('hint')).toBeVisible();
		await point('a0_b2', 2).click();
		await point('a1_b1', 2).click();
		await page.getByRole('button', { name: 'Formar célula' }).click();
		await expect.element(page.getByTestId('hint')).not.toBeInTheDocument();
	});

	it('the complete circuit summary lists the help used (AJU-07)', async () => {
		await render(Compression, { session: r2BeforeTheCpa(), view: 'dots', ...R1_WIDTH });
		await page.getByRole('button', { name: 'Dica' }).click();
		await show().click();
		await page.getByRole('button', { name: 'Limite inferior' }).click();
		await show().click();
		await page.getByRole('button', { name: 'Fechar com CPA' }).click();
		await expect
			.element(page.getByTestId('help-log'))
			.toHaveTextContent('Ajudas usadas: dicas: 1 · limite inferior: 1 (conta: 0)');
	});

	it('with no help used, the summary says so (AJU-07)', async () => {
		await render(Compression, { session: r2BeforeTheCpa(), view: 'dots', ...R1_WIDTH });
		await page.getByRole('button', { name: 'Fechar com CPA' }).click();
		await expect.element(page.getByTestId('help-log')).toHaveTextContent('Ajudas usadas: nenhuma.');
	});
});
