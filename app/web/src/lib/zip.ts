// Caminho relativo: app/web/src/lib/zip.ts
// A minimal .zip writer (stored, no compression) for the VHDL files (VHD-05). The files are
// a few tens of kilobytes of text, so compression would not pay for a dependency.

export interface ZipEntry {
	name: string;
	contents: string;
}

const CRC_TABLE = (() => {
	const t = new Uint32Array(256);
	for (let n = 0; n < 256; n++) {
		let c = n;
		for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
		t[n] = c >>> 0;
	}
	return t;
})();

/** CRC-32 (IEEE 802.3), as the zip format requires. */
export function crc32(bytes: Uint8Array): number {
	let c = 0xffffffff;
	for (const b of bytes) c = CRC_TABLE[(c ^ b) & 0xff] ^ (c >>> 8);
	return (c ^ 0xffffffff) >>> 0;
}

/** Local date and time in the MS-DOS format of zip headers (2-second resolution). */
function dosDateTime(d: Date): { time: number; date: number } {
	return {
		time: (d.getHours() << 11) | (d.getMinutes() << 5) | (d.getSeconds() >> 1),
		date: ((d.getFullYear() - 1980) << 9) | ((d.getMonth() + 1) << 5) | d.getDate()
	};
}

/** Builds a .zip with every entry stored as is (APPNOTE 4.3: local headers, central
 *  directory, end record). Names are UTF-8 (flag bit 11). */
export function zip(entries: ZipEntry[], date: Date = new Date()): Uint8Array<ArrayBuffer> {
	const enc = new TextEncoder();
	const { time, date: day } = dosDateTime(date);
	const UTF8 = 0x0800;
	const local: Uint8Array[] = [];
	const central: Uint8Array[] = [];
	let offset = 0;
	for (const e of entries) {
		const name = enc.encode(e.name);
		const data = enc.encode(e.contents);
		const crc = crc32(data);

		const lh = new Uint8Array(30 + name.length);
		const l = new DataView(lh.buffer);
		l.setUint32(0, 0x04034b50, true);
		l.setUint16(4, 10, true); // version needed: 1.0, stored
		l.setUint16(6, UTF8, true);
		l.setUint16(8, 0, true); // stored
		l.setUint16(10, time, true);
		l.setUint16(12, day, true);
		l.setUint32(14, crc, true);
		l.setUint32(18, data.length, true);
		l.setUint32(22, data.length, true);
		l.setUint16(26, name.length, true);
		l.setUint16(28, 0, true);
		lh.set(name, 30);

		const ch = new Uint8Array(46 + name.length);
		const c = new DataView(ch.buffer);
		c.setUint32(0, 0x02014b50, true);
		c.setUint16(4, 20, true); // made by: version 2.0
		c.setUint16(6, 10, true);
		c.setUint16(8, UTF8, true);
		c.setUint16(10, 0, true);
		c.setUint16(12, time, true);
		c.setUint16(14, day, true);
		c.setUint32(16, crc, true);
		c.setUint32(20, data.length, true);
		c.setUint32(24, data.length, true);
		c.setUint16(28, name.length, true);
		// extra, comment, disk, internal and external attributes stay 0
		c.setUint32(42, offset, true);
		ch.set(name, 46);

		local.push(lh, data);
		central.push(ch);
		offset += lh.length + data.length;
	}
	const centralSize = central.reduce((n, b) => n + b.length, 0);
	const end = new Uint8Array(22);
	const v = new DataView(end.buffer);
	v.setUint32(0, 0x06054b50, true);
	v.setUint16(8, entries.length, true);
	v.setUint16(10, entries.length, true);
	v.setUint32(12, centralSize, true);
	v.setUint32(16, offset, true);

	const out = new Uint8Array(offset + centralSize + end.length);
	let at = 0;
	for (const part of [...local, ...central, end]) {
		out.set(part, at);
		at += part.length;
	}
	return out;
}
