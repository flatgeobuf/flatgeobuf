import { beforeAll, afterAll, describe, it, expect } from 'vitest';
import { HttpReader } from './http-reader';
import { fromFeature } from './geojson/feature';
import LocalWebServer from 'local-web-server';

describe('http reader', () => {
    let lws: LocalWebServer;
    beforeAll(async () => {
        lws = await LocalWebServer.create();
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
        let reader = await HttpReader.open(testUrl);

        let features = [];
        for await (const feature of reader.selectBbox(rect)) {
            features.push(fromFeature(feature, reader.header));
        }
        expect(features.length).toBe(86);
        let actual = features
            .slice(0, 4)
            .map((f) => `${f.properties.NAME}, ${f.properties.STATE}`);
        let expected = [
            'Cheyenne, KS',
            'Rawlins, KS',
            'Yuma, CO',
            'Washington, CO',
        ];
        expect(actual).toEqual(expected);
    });
});
