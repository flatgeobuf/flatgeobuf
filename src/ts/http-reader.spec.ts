import Lws from 'lws';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';
import { fromFeature, type IGeoJsonFeature } from './geojson/feature';
import { HttpReader } from './http-reader';

describe('http reader', () => {
    let lws: Lws;
    beforeAll(async () => {
        lws = await Lws.create({ stack: ['lws-range', 'lws-static'] });
    });
    afterAll(() => {
        if (lws) lws.server.close();
    });

    it('fetches a subset of data based on bounding box', async () => {
        const testUrl = `http://localhost:${lws.config.port}/test/data/UScounties.fgb`;
        const rect = {
            minX: -106.88,
            minY: 36.75,
            maxX: -101.11,
            maxY: 41.24,
        };
        const reader = await HttpReader.open(testUrl, false);

        const features: IGeoJsonFeature[] = [];
        for await (const feature of reader.selectBbox(rect)) {
            features.push(fromFeature(feature.id, feature.feature, reader.header));
        }
        expect(features.length).toBe(86);
        const actual = features.slice(0, 4).map((f) => `${f.properties?.NAME}, ${f.properties?.STATE}`);
        const expected = ['Cheyenne, KS', 'Rawlins, KS', 'Yuma, CO', 'Washington, CO'];
        expect(actual).toEqual(expected);
    });

    it('can fetch the final feature', async () => {
        const testUrl = `http://localhost:${lws.config.port}/test/data/countries.fgb`;
        const rect = {
            minX: -61.2,
            minY: -51.85,
            maxX: -60.0,
            maxY: -51.25,
        };
        const reader = await HttpReader.open(testUrl, true);
        expect(179).toBe(reader.header.featuresCount);

        let featureCount = 0;
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
        for await (const _feature of reader.selectBbox(rect)) {
            featureCount++;
        }
        expect(featureCount).toBe(2);
    });
});
