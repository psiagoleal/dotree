// Caminho relativo: app/web/src/lib/keys.spec.ts
import { describe, expect, it } from 'vitest';
import { keyAction } from './keys';

const key = (k: string, target: Partial<HTMLElement> | null = null, mods = {}) =>
	({
		key: k,
		target,
		ctrlKey: false,
		metaKey: false,
		altKey: false,
		shiftKey: false,
		...mods
	}) as never;

describe('keyAction (VIS-03, GER-07)', () => {
	it('maps the plain keys', () => {
		expect(keyAction(key('v'))).toBe('toggleView');
		expect(keyAction(key('V'))).toBe('toggleView');
		expect(keyAction(key('f'))).toBe('form');
		expect(keyAction(key('Escape'))).toBe('clearSelection');
		expect(keyAction(key('b'))).toBeNull();
	});

	it('maps undo and redo with Ctrl or Cmd', () => {
		expect(keyAction(key('z', null, { ctrlKey: true }))).toBe('undo');
		expect(keyAction(key('z', null, { metaKey: true }))).toBe('undo');
		expect(keyAction(key('Z', null, { ctrlKey: true, shiftKey: true }))).toBe('redo');
		expect(keyAction(key('y', null, { ctrlKey: true }))).toBe('redo');
	});

	it('never fires while typing (V, F are variable names; Ctrl+Z undoes text)', () => {
		for (const tagName of ['INPUT', 'TEXTAREA', 'SELECT']) {
			expect(keyAction(key('V', { tagName }))).toBeNull();
			expect(keyAction(key('z', { tagName }, { ctrlKey: true }))).toBeNull();
		}
		expect(keyAction(key('f', { tagName: 'DIV', isContentEditable: true }))).toBeNull();
		expect(keyAction(key('v', { tagName: 'BUTTON' }))).toBe('toggleView');
	});

	it('plain keys with a modifier are left to the browser', () => {
		expect(keyAction(key('v', null, { ctrlKey: true }))).toBeNull();
		expect(keyAction(key('f', null, { altKey: true }))).toBeNull();
		expect(keyAction(key('v', null, { metaKey: true }))).toBeNull();
	});
});
