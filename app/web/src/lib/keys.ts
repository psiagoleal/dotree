// Caminho relativo: app/web/src/lib/keys.ts
// Keyboard shortcuts of the page (VIS-03, GER-07). None fires inside a text field: V and F
// are variable names, and Ctrl+Z there undoes the text.

export type KeyAction = 'toggleView' | 'form' | 'clearSelection' | 'undo' | 'redo';

const TYPING = new Set(['INPUT', 'TEXTAREA', 'SELECT']);

export function keyAction(e: KeyboardEvent): KeyAction | null {
	const target = e.target as HTMLElement | null;
	if (target && (TYPING.has(target.tagName) || target.isContentEditable)) return null;
	const key = e.key.toLowerCase();
	if (e.ctrlKey || e.metaKey) {
		if (e.altKey) return null;
		if (key === 'z') return e.shiftKey ? 'redo' : 'undo';
		if (key === 'y') return 'redo';
		return null;
	}
	if (e.altKey) return null;
	if (key === 'v') return 'toggleView';
	if (key === 'f') return 'form';
	if (key === 'escape') return 'clearSelection';
	return null;
}
