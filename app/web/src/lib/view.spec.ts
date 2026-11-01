// Caminho relativo: app/web/src/lib/view.spec.ts
import { describe, expect, it } from 'vitest';
import { VIEW_STORAGE_KEY, isViewShortcut, readView, writeView } from './view';

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

describe('isViewShortcut (VIS-03)', () => {
	const key = (k: string, target: Partial<HTMLElement> | null = null, mods = {}) =>
		({ key: k, target, ctrlKey: false, metaKey: false, altKey: false, ...mods }) as never;

	it('is the V key, upper or lower case', () => {
		expect(isViewShortcut(key('v'))).toBe(true);
		expect(isViewShortcut(key('V'))).toBe(true);
		expect(isViewShortcut(key('b'))).toBe(false);
	});

	it('never fires while typing (V is a variable name) or with a modifier', () => {
		expect(isViewShortcut(key('V', { tagName: 'INPUT' }))).toBe(false);
		expect(isViewShortcut(key('v', { tagName: 'TEXTAREA' }))).toBe(false);
		expect(isViewShortcut(key('v', { tagName: 'SELECT' }))).toBe(false);
		expect(isViewShortcut(key('v', { tagName: 'DIV', isContentEditable: true }))).toBe(false);
		expect(isViewShortcut(key('v', null, { ctrlKey: true }))).toBe(false);
		expect(isViewShortcut(key('v', null, { metaKey: true }))).toBe(false);
		expect(isViewShortcut(key('v', null, { altKey: true }))).toBe(false);
		expect(isViewShortcut(key('v', { tagName: 'BUTTON' }))).toBe(true);
	});
});
