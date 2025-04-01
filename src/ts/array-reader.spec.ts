import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { ArrayReader } from './array-reader.js';
import { type IGeoJsonFeature, fromFeature } from './geojson/feature.js';
import type { Rect } from './packedrtree.js';

describe('ArrayReader', () => {
    it('Should filter features using ArrayReader', async () => {
        const bytes = new Uint8Array(readFileSync(path.join(__dirname, '../../test/data/UScounties.fgb')));

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
});
