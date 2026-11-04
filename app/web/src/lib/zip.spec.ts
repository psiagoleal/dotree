// Caminho relativo: app/web/src/lib/zip.spec.ts
import { describe, expect, it } from 'vitest';
import { crc32 as zlibCrc32 } from 'node:zlib';
import { crc32, zip } from './zip.ts';

const enc = new TextEncoder();

describe('crc32', () => {
	it('matches the standard check value and node:zlib', () => {
		expect(crc32(enc.encode('123456789'))).toBe(0xcbf43926);
		expect(crc32(new Uint8Array())).toBe(0);
		const data = Uint8Array.from({ length: 1000 }, (_, i) => (i * 7919) % 256);
		expect(crc32(data)).toBe(zlibCrc32(data));
	});
});

describe('zip', () => {
	// Reads the archive back by the format's fixed offsets (APPNOTE 4.3).
	const date = new Date(2026, 10, 4, 13, 45, 30);
	const files = [
		{ name: 'full_adder.vhd', contents: 'entity full_adder is\n' },
		{ name: 'vazio.vhd', contents: '' }
	];
	const bytes = zip(files, date);
	const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
	const u16 = (at: number) => view.getUint16(at, true);
	const u32 = (at: number) => view.getUint32(at, true);
	const text = (at: number, n: number) => new TextDecoder().decode(bytes.subarray(at, at + n));

	it('ends with a directory of every file', () => {
		const end = bytes.length - 22;
		expect(u32(end)).toBe(0x06054b50);
		expect(u16(end + 8)).toBe(2);
		expect(u16(end + 10)).toBe(2);
		const size = u32(end + 12);
		const offset = u32(end + 16);
		expect(offset + size).toBe(end);
		let at = offset;
		for (const f of files) {
			const data = enc.encode(f.contents);
			expect(u32(at)).toBe(0x02014b50);
			expect(u16(at + 10)).toBe(0); // stored
			expect(u32(at + 16)).toBe(crc32(data));
			expect(u32(at + 20)).toBe(data.length);
			expect(u32(at + 24)).toBe(data.length);
			const nameLength = u16(at + 28);
			expect(text(at + 46, nameLength)).toBe(f.name);
			// The local header it points to holds the same name and the data.
			const local = u32(at + 42);
			expect(u32(local)).toBe(0x04034b50);
			expect(u32(local + 14)).toBe(crc32(data));
			expect(text(local + 30, u16(local + 26))).toBe(f.name);
			expect(text(local + 30 + u16(local + 26) + u16(local + 28), data.length)).toBe(f.contents);
			at += 46 + nameLength + u16(at + 30) + u16(at + 32);
		}
		expect(at).toBe(end);
	});

	it('stamps the files with the given local date (MS-DOS format)', () => {
		const time = (13 << 11) | (45 << 5) | 15;
		const day = ((2026 - 1980) << 9) | (11 << 5) | 4;
		expect(u16(10)).toBe(time);
		expect(u16(12)).toBe(day);
	});
});
