// Caminho relativo: app/web/src/lib/persist.spec.ts
// The exercise in local storage (SAL-01, SAL-04, SAL-05): reading, writing, failing quietly,
// and saving after every change of the session.
import { describe, expect, it, vi } from 'vitest';
import type { CompressionSession } from '#lib/core.ts';
import { EXERCISE_STORAGE_KEY, autosaving, clearSaved, loadSaved, saveExercise } from './persist';

function memory(): Storage {
	const data = new Map<string, string>();
	return {
		get length() {
			return data.size;
		},
		clear: () => data.clear(),
		getItem: (k) => data.get(k) ?? null,
		key: (i) => [...data.keys()][i] ?? null,
		removeItem: (k) => void data.delete(k),
		setItem: (k, v) => void data.set(k, String(v))
	};
}

const throwing = (): Storage => {
	const s = memory();
	const fail = () => {
		throw new DOMException('quota', 'QuotaExceededError');
	};
	return { ...s, getItem: fail, setItem: fail, removeItem: fail };
};

describe('local storage of the exercise', () => {
	it('round-trips the text under its own key', () => {
		const s = memory();
		expect(loadSaved(s)).toEqual({ kind: 'none' });
		expect(saveExercise(s, '{"a":1}')).toBe(true);
		expect(s.getItem(EXERCISE_STORAGE_KEY)).toBe('{"a":1}');
		expect(loadSaved(s)).toEqual({ kind: 'found', text: '{"a":1}' });
		clearSaved(s);
		expect(loadSaved(s)).toEqual({ kind: 'none' });
	});

	it('reports a missing, blocked or full storage instead of throwing (SAL-04)', () => {
		expect(loadSaved(undefined)).toEqual({ kind: 'unavailable' });
		expect(saveExercise(undefined, 'x')).toBe(false);
		expect(loadSaved(throwing())).toEqual({ kind: 'unavailable' });
		expect(saveExercise(throwing(), 'x')).toBe(false);
		expect(() => clearSaved(throwing())).not.toThrow();
	});
});

describe('autosaving', () => {
	function fake() {
		let n = 0;
		const calls: string[] = [];
		const session = new Proxy({} as CompressionSession, {
			get: (_, key: string) =>
				key === 'exercise'
					? () => `state ${n}`
					: (...args: unknown[]) => {
							calls.push(key);
							if (key !== 'snapshot' && key !== 'helpLog') n++;
							return args[0] ?? key;
						}
		});
		return { session, calls };
	}

	it('saves after every action and reveal, with the result passed through', () => {
		const { session } = fake();
		const save = vi.fn();
		const s = autosaving(session, save);
		expect(s.formCell([1, 2, 3])).toEqual([1, 2, 3]);
		s.concludeLevel();
		s.closeWithCpa();
		s.goToLevel(1);
		s.undo();
		s.redo();
		s.hint();
		s.nextMove();
		s.lowerBound(false);
		s.solution(true);
		s.revealWidth(7);
		s.revealWidthPath();
		s.applySolution();
		expect(save).toHaveBeenCalledTimes(13);
		expect(save).toHaveBeenLastCalledWith('state 13');
	});

	it('does not save on reads', () => {
		const { session, calls } = fake();
		const save = vi.fn();
		const s = autosaving(session, save);
		s.snapshot();
		s.helpLog();
		expect(calls).toEqual(['snapshot', 'helpLog']);
		expect(save).not.toHaveBeenCalled();
	});
});
