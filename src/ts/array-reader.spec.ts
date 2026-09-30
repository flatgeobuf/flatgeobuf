import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it, vi } from 'vitest';
import { ArrayReader } from './array-reader.js';
import { fromFeature, type IGeoJsonFeature } from './geojson/feature.js';
import type { Rect } from './packedrtree.js';

const file = readFileSync(path.join(__dirname, '../../test/data/UScounties.fgb'));
const padded = new Uint8Array(file.byteLength + 8);
padded.set(file, 8);

describe('ArrayReader', () => {
    it.each([
        { input: 'Uint8Array', bytes: new Uint8Array(file) },
        { input: 'Node Buffer', bytes: file },
        { input: 'Uint8Array with a byteOffset', bytes: padded.subarray(8) },
    ])('Should filter features from a $input', async ({ bytes }) => {
        const rect: Rect = {
            minX: -106.88,
            minY: 36.75,
            maxX: -101.11,
            maxY: 41.24,
        };

        const reader = ArrayReader.open(bytes);

        const features: IGeoJsonFeature[] = [];
        for await (const feature of reader.selectBbox(rect)) {
            features.push(fromFeature(feature.id, feature.feature, reader.header));
        }

        expect(features.length).toBe(86);
        const actual = features.slice(0, 4).map((f) => `${f.properties?.NAME}, ${f.properties?.STATE}`);
        const expected = ['Texas, OK', 'Cimarron, OK', 'Taos, NM', 'Colfax, NM'];
        expect(actual).toEqual(expected);
    });

    it('Should search the index without logging', async () => {
        const debug = vi.spyOn(console, 'debug').mockImplementation(() => {});
        const reader = ArrayReader.open(new Uint8Array(file));
        for await (const _ of reader.selectBbox({ minX: -125, minY: 24, maxX: -66, maxY: 50 })) {
        }
        expect(debug).not.toHaveBeenCalled();
        debug.mockRestore();
    });
});
