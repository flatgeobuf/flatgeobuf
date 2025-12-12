import { createServer, type Server } from 'node:http';
import type { AddressInfo } from 'node:net';
import sirv from 'sirv';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';
import { fromFeature, type IGeoJsonFeature } from './geojson/feature';
import { HttpReader } from './http-reader';

describe('http reader', () => {
    let server: Server;
    let port: number;

    beforeAll(() => {
        const serve = sirv('.', { dev: true, single: false });
        server = createServer((req, res) => serve(req, res));
        return new Promise<void>((resolve) => {
            server.listen(0, () => {
                port = (server.address() as AddressInfo).port;
                resolve();
            });
        });
    });

    afterAll(() => new Promise<void>((resolve) => server?.close(() => resolve())));

    it('fetches a subset of data based on bounding box', async () => {
        const testUrl = `http://localhost:${port}/test/data/UScounties.fgb`;
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
        const expected = ['Texas, OK', 'Cimarron, OK', 'Taos, NM', 'Colfax, NM'];
        expect(actual).toEqual(expected);
    });

    it('can fetch the final feature', async () => {
        const testUrl = `http://localhost:${port}/test/data/countries.fgb`;
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
