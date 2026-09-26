import { readFileSync } from 'node:fs';
import { createServer, type Server } from 'node:http';
import type { AddressInfo } from 'node:net';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';
import type { IGeoJsonFeature } from './geojson/feature.js';
import { deserialize } from './geojson.js';
import { SeekableZstdReader } from './seekable-zstd.js';
import { takeAsync } from './streams/utils.js';

const seekable32KiB = new Uint8Array(readFileSync('test/data/countries-32k.fgb.zst'));
const seekable64KiB = new Uint8Array(readFileSync('test/data/countries-64k.fgb.zst'));

describe('seekable Zstandard', () => {
    let server: Server;
    let port: number;
    const requestedRanges: string[] = [];

    beforeAll(async () => {
        server = createServer((request, response) => {
            const range = request.headers.range;
            if (!range) {
                response.writeHead(400).end();
                return;
            }
            const suffixMatch = range.match(/^bytes=-(\d+)$/);
            const rangeMatch = range.match(/^bytes=(\d+)-(\d+)$/);
            if (!suffixMatch && !rangeMatch) {
                response.writeHead(416).end();
                return;
            }
            const start = suffixMatch
                ? Math.max(seekable32KiB.byteLength - Number(suffixMatch[1]), 0)
                : Number(rangeMatch?.[1]);
            const end = suffixMatch
                ? seekable32KiB.byteLength - 1
                : Math.min(Number(rangeMatch?.[2]), seekable32KiB.byteLength - 1);
            if (start > end || start >= seekable32KiB.byteLength) {
                response.writeHead(416).end();
                return;
            }
            requestedRanges.push(range);
            response.writeHead(206, {
                'Accept-Ranges': 'bytes',
                'Content-Length': end - start + 1,
                'Content-Range': `bytes ${start}-${end}/${seekable32KiB.byteLength}`,
            });
            response.end(seekable32KiB.subarray(start, end + 1));
        });
        await new Promise<void>((resolve) => {
            server.listen(0, () => {
                port = (server.address() as AddressInfo).port;
                resolve();
            });
        });
    });

    afterAll(() => new Promise<void>((resolve) => server?.close(() => resolve())));

    it.each([
        { target: '32 KiB', bytes: seekable32KiB },
        { target: '64 KiB', bytes: seekable64KiB },
    ])('reads all features from the t2sz $target fixture', async ({ bytes }) => {
        const features = await takeAsync<IGeoJsonFeature>(deserialize(bytes, { seekableZstd: true }));
        expect(features).toHaveLength(179);
    });

    it('reads indexed bbox results from an externally produced wrapper', async () => {
        const rect = { minX: -61.2, minY: -51.85, maxX: -60.0, maxY: -51.25 };
        const features = await takeAsync<IGeoJsonFeature>(deserialize(seekable32KiB, { rect, seekableZstd: true }));
        expect(features).toHaveLength(2);
    });

    it('reads indexed bbox results from a range-enabled URL without fetching the full file', async () => {
        requestedRanges.length = 0;
        const rect = { minX: -61.2, minY: -51.85, maxX: -60.0, maxY: -51.25 };
        const features = await takeAsync<IGeoJsonFeature>(
            deserialize(`http://localhost:${port}/countries.fgb.zst`, { rect, seekableZstd: true }),
        );
        expect(features).toHaveLength(2);
        expect(requestedRanges.length).toBeGreaterThan(3);
        expect(requestedRanges[0]).toBe('bytes=-9');
        const footer = new DataView(seekable32KiB.buffer, seekable32KiB.byteOffset + seekable32KiB.byteLength - 9, 9);
        const entrySize = 8 + ((footer.getUint8(4) & 0x80) !== 0 ? 4 : 0);
        const tableLength = 8 + footer.getUint32(0, true) * entrySize + 9;
        expect(requestedRanges[1]).toBe(
            `bytes=${seekable32KiB.byteLength - tableLength}-${seekable32KiB.byteLength - 1}`,
        );
        const requestedBytes = requestedRanges.reduce((total, range) => {
            const suffix = range.match(/^bytes=-(\d+)$/);
            if (suffix) return total + Math.min(Number(suffix[1]), seekable32KiB.byteLength);
            const [start, end] = range.slice('bytes='.length).split('-').map(Number);
            return total + end - start + 1;
        }, 0);
        expect(requestedBytes).toBeLessThan(seekable32KiB.byteLength);
    });

    it('rejects invalid seek tables', async () => {
        const corrupted = seekable32KiB.slice();
        corrupted[corrupted.byteLength - 1] ^= 0xff;
        await expect(SeekableZstdReader.open(corrupted)).rejects.toThrow('missing seekable magic');
    });
});
