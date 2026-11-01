// Caminho relativo: app/web/src/lib/view.spec.ts
import { describe, expect, it } from 'vitest';
import { VIEW_STORAGE_KEY, readView, writeView } from './view';

function memory(): Storage {
	const data = new Map<string, string>();
	return {
		getItem: (k) => data.get(k) ?? null,
		setItem: (k, v) => void data.set(k, v),
		removeItem: (k) => void data.delete(k),
		clear: () => data.clear(),
		key: () => null,
		get length() {
			return data.size;
		}
	};
}

const throwing = {
	getItem: () => {
		throw new Error('blocked');
	},
	setItem: () => {
		throw new Error('blocked');
	}
};

describe('view preference (VIS-03)', () => {
	it('defaults to dots and remembers the last choice', () => {
		const s = memory();
		expect(readView(s)).toBe('dots');
		writeView(s, 'indexed');
		expect(s.getItem(VIEW_STORAGE_KEY)).toBe('indexed');
		expect(readView(s)).toBe('indexed');
	});

	it('ignores junk and blocked storage', () => {
		const s = memory();
		s.setItem(VIEW_STORAGE_KEY, 'squares');
		expect(readView(s)).toBe('dots');
		expect(readView(throwing)).toBe('dots');
		expect(readView(undefined)).toBe('dots');
		expect(() => writeView(throwing, 'indexed')).not.toThrow();
	});
});
