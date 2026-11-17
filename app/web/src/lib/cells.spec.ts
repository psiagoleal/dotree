// Caminho relativo: app/web/src/lib/cells.spec.ts
import { describe, expect, it } from 'vitest';
import { cellName, instanceName, kindText } from './cells';

describe('cell names (CMP-20)', () => {
	it('names counters like the core and the messages', () => {
		expect(instanceName('fa', 3)).toBe('FA3');
		expect(instanceName('c73', 3)).toBe('C73_3');
		expect(instanceName('c154', 12)).toBe('C154_12');
		expect(kindText('ha')).toBe('HA');
		expect(kindText('c154')).toBe('contador 15:4');
		expect(cellName({ kind: 'fa', number: 2 })).toBe('FA 2');
		expect(cellName({ kind: 'c73', number: 5 })).toBe('contador 7:3, célula 5');
	});
});
