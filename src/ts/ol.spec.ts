import { readFileSync } from 'node:fs';
import Feature, { type FeatureLike } from 'ol/Feature.js';
import GeoJSON from 'ol/format/GeoJSON.js';
import WKT from 'ol/format/WKT.js';
import type Geometry from 'ol/geom/Geometry.js';
import type Point from 'ol/geom/Point.js';
import type SimpleGeometry from 'ol/geom/SimpleGeometry.js';
import { transform } from 'ol/proj';
import type RenderFeature from 'ol/render/Feature.js';
import { describe, expect, it } from 'vitest';
import { deserialize, serialize } from './ol.js';
import { arrayToStream, takeAsync } from './streams/utils.js';

const format = new WKT();
const geojson = new GeoJSON();

const g = (features: Array<Feature<Geometry>>) => geojson.writeFeatures(features);

function makeFeatureCollection(wkt: string /*, properties?: any*/) {
    return makeFeatureCollectionFromArray([wkt] /*, properties*/);
}

function makeFeatureCollectionFromArray(wkts: string[] /*, properties?: any*/): Feature[] {
    const geometries = wkts.map((wkt) => format.readGeometry(wkt));
    const features = geometries.map((geometry, index) => {
        const f = new Feature({ geometry });
        f.setId(index);
        return f;
    });
    return features;
}

describe('ol module', () => {
    describe('Geometry roundtrips', () => {
        it('Point', async () => {
            const expected = makeFeatureCollection('POINT(1.2 -2.1)');
            const s = serialize(expected);
            const actual = (await takeAsync<FeatureLike>(deserialize(s))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('Point via stream', async () => {
            const expected = makeFeatureCollection('POINT(1.2 -2.1)');
            const s = serialize(expected);
            const stream = arrayToStream(s);
            const actual = (await takeAsync<FeatureLike>(deserialize(stream))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('Point Z', async () => {
            const expected = makeFeatureCollection('POINT Z(1.2 -2.1 5)');
            const s = serialize(expected);
            const actual = (await takeAsync<FeatureLike>(deserialize(s))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('Point M', async () => {
            const expected = makeFeatureCollection('POINT M(1.2 -2.1 5)');
            const s = serialize(expected);
            const actual = (await takeAsync<FeatureLike>(deserialize(s))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('Point ZM', async () => {
            const expected = makeFeatureCollection('POINT ZM(1.2 -2.1 5 12.8)');
            const s = serialize(expected);
            const actual = (await takeAsync<FeatureLike>(deserialize(s))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('Points', async () => {
            const expected = makeFeatureCollectionFromArray(['POINT(1.2 -2.1)', 'POINT(2.4 -4.8)']);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('Points Z', async () => {
            const expected = makeFeatureCollectionFromArray(['POINT Z(1.2 -2.1 5)', 'POINT Z(2.4 -4.8 5.6)']);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiPoint', async () => {
            const expected = makeFeatureCollection('MULTIPOINT(10 40, 40 30, 20 20, 30 10)');
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiPoint Z', async () => {
            const expected = makeFeatureCollection('MULTIPOINT Z(10 40 0.5, 40 30 0.5, 20 20 0.5, 30 10 0.5)');
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiPoint ZM', async () => {
            const expected = makeFeatureCollection('MULTIPOINT ZM(10 40 0.5 1, 40 30 0.5 1, 20 20 0.5 1, 30 10 0.5 1)');
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('LineString', async () => {
            const expected = makeFeatureCollection('LINESTRING(1.2 -2.1, 2.4 -4.8)');
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('LineString Z', async () => {
            const expected = makeFeatureCollection('LINESTRING Z(1.2 -2.1 5, 2.4 -4.8 0.5)');
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('LineString ZM', async () => {
            const expected = makeFeatureCollection('LINESTRING ZM(1.2 -2.1 5 1, 2.4 -4.8 0.5 1)');
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiLineString', async () => {
            const expected = makeFeatureCollection(`MULTILINESTRING((10 10, 20 20, 10 40),
 (40 40, 30 30, 40 20, 30 10), (50 50, 60 60, 50 90))`);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiLineString Z', async () => {
            const expected = makeFeatureCollection(`MULTILINESTRING Z((10 10 5, 20 20 0.5, 10 40 12.1),
 (40 40 4, 30 30 3, 40 20 2, 30 10 1), (50 50 5, 60 60 6, 50 90 9))`);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiLineString ZM', async () => {
            const expected = makeFeatureCollection(`MULTILINESTRING ZM((10 10 5 1, 20 20 0.5 1, 10 40 12.1 1),
 (40 40 4 1, 30 30 3 1, 40 20 2 1, 30 10 1 2), (50 50 5 1, 60 60 6 1, 50 90 9 1))`);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiLineStringSinglePart', async () => {
            const expected = makeFeatureCollection('MULTILINESTRING((1.2 -2.1, 2.4 -4.8))');
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('Polygon', async () => {
            const expected = makeFeatureCollection('POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))');
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('Polygon Z', async () => {
            const expected = makeFeatureCollection('POLYGON Z((30 10 12.1, 40 40 4, 20 40 4, 10 20 0.5, 30 10 12))');
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('Polygon ZM', async () => {
            const expected = makeFeatureCollection(
                'POLYGON ZM((30 10 12.1 1, 40 40 4 1, 20 40 4 1, 10 20 0.5 1, 30 10 12 1))',
            );
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('PolygonWithHole', async () => {
            const expected = makeFeatureCollection(`POLYGON ((35 10, 45 45, 15 40, 10 20, 35 10),
 (20 30, 35 35, 30 20, 20 30))`);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('PolygonWithHole Z', async () => {
            const expected = makeFeatureCollection(`POLYGON Z ((35 10 4, 45 45 4, 15 40 4, 10 20 4, 35 10 4),
 (20 30 4, 35 35 4, 30 20 4, 20 30 4))`);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('PolygonWithHole ZM', async () => {
            const expected = makeFeatureCollection(`POLYGON ZM ((35 10 4 1, 45 45 4 1, 15 40 4 1, 10 20 4 1, 35 10 4 1),
 (20 30 4 1, 35 35 4 1, 30 20 4 1, 20 30 4 1))`);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('PolygonWithTwoHoles', async () => {
            const expected = makeFeatureCollection(`POLYGON ((35 10, 45 45, 15 40, 10 20, 35 10),
 (20 30, 35 35, 30 20, 20 30), (20 30, 35 35, 30 20, 20 30))`);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiPolygon', async () => {
            const expected = makeFeatureCollection(`MULTIPOLYGON (((30 20, 45 40, 10 40, 30 20)),
 ((15 5, 40 10, 10 20, 5 10, 15 5)))`);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            // should encode into 18 flat coords, ends [8, 16] endss [1, 1]
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiPolygon Z', async () => {
            const expected = makeFeatureCollection(`MULTIPOLYGON Z (((30 20 1, 45 40 3, 10 40 4, 30 20 6)),
 ((15 5 1, 40 10 1, 10 20 1, 5 10 1, 15 5 1)))`);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            // should encode into 18 flat coords, ends [8, 16] endss [1, 1]
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiPolygon ZM', async () => {
            const expected = makeFeatureCollection(`MULTIPOLYGON ZM (((30 20 1 2, 45 40 3 2, 10 40 4 2, 30 20 6 2)),
 ((15 5 1 2, 40 10 1 2, 10 20 1 2, 5 10 1 2, 15 5 1 2)))`);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            // should encode into 18 flat coords, ends [8, 16] endss [1, 1]
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiPolygonWithHole', async () => {
            const expected = makeFeatureCollection(`MULTIPOLYGON (((40 40, 20 45, 45 30, 40 40)),
 ((20 35, 10 30, 10 10, 30 5, 45 20, 20 35), (30 20, 20 15, 20 25, 30 20)))`);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            // NOTE: 28 flat coords, ends = [4, 10, 14], endss = [1, 2]
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiPolygonWithHole Z', async () => {
            const expected = makeFeatureCollection(`MULTIPOLYGON Z(((40 40 1, 20 45 1, 45 30 1, 40 40 1)),
 ((20 35 1, 10 30 1, 10 10 1, 30 5 1, 45 20 1, 20 35 1), (30 20 1, 20 15 1, 20 25 1, 30 20 1)))`);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            // NOTE: 28 flat coords, ends = [4, 10, 14], endss = [1, 2]
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiPolygonWithHole ZM', async () => {
            const expected = makeFeatureCollection(`MULTIPOLYGON ZM (((40 40 1 2, 20 45 1 2, 45 30 1 2, 40 40 1 2)),
 ((20 35 1 2, 10 30 1 2, 10 10 1 2, 30 5 1 2, 45 20 1 2, 20 35 1 2), (30 20 1 2, 20 15 1 2, 20 25 1 2, 30 20 1 2)))`);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            // NOTE: 28 flat coords, ends = [4, 10, 14], endss = [1, 2]
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiPolygonSinglePart', async () => {
            const expected = makeFeatureCollection('MULTIPOLYGON (((30 20, 45 40, 10 40, 30 20)))');
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiPolygonSinglePartWithHole', async () => {
            const expected = makeFeatureCollection(`MULTIPOLYGON (((35 10, 45 45, 15 40, 10 20, 35 10),
 (20 30, 35 35, 30 20, 20 30))))`);
            const actual = (await takeAsync<FeatureLike>(deserialize(serialize(expected)))) as Feature[];
            expect(g(actual)).to.equal(g(expected));
        });

        it('MultiPolygon with two holes', async () => {
            const expected = {
                type: 'FeatureCollection',
                features: [
                    {
                        type: 'Feature',
                        id: 0,
                        properties: { test: 1 },
                        geometry: {
                            type: 'MultiPolygon',
                            coordinates: [
                                [
                                    [
                                        [102.0, 2.0],
                                        [103.0, 2.0],
                                        [103.0, 3.0],
                                        [102.0, 3.0],
                                        [102.0, 2.0],
                                    ],
                                ],
                                [
                                    [
                                        [100.0, 0.0],
                                        [101.0, 0.0],
                                        [101.0, 1.0],
                                        [100.0, 1.0],
                                        [100.0, 0.0],
                                    ],
                                    [
                                        [100.2, 0.2],
                                        [100.8, 0.2],
                                        [100.8, 0.8],
                                        [100.2, 0.8],
                                        [100.2, 0.2],
                                    ],
                                ],
                            ],
                        },
                    },
                ],
            };
            const actual = (await takeAsync<FeatureLike>(
                deserialize(serialize(geojson.readFeatures(expected))),
            )) as Feature[];
            expect(JSON.parse(g(actual))).to.deep.equal(expected);
        });

        it('Should parse UScounties fgb produced from GDAL array buffer', async () => {
            const buffer = readFileSync('./test/data/UScounties.fgb');
            const bytes = new Uint8Array(buffer);
            const features = (await takeAsync<FeatureLike>(deserialize(bytes))) as Feature[];
            expect(features.length).to.eq(3221);
            for (const f of features)
                expect((f.getGeometry() as SimpleGeometry).getCoordinates()?.length).to.be.greaterThan(0);
        });

        it('Should parse countries fgb produced from GDAL array buffer', async () => {
            const buffer = readFileSync('./test/data/countries.fgb');
            const bytes = new Uint8Array(buffer);
            const features = (await takeAsync<FeatureLike>(deserialize(bytes))) as Feature[];
            expect(features.length).to.eq(179);
            for (const f of features)
                expect((f.getGeometry() as SimpleGeometry).getCoordinates()?.length).to.be.greaterThan(0);
        });

        it('Should parse countries fgb produced from GDAL stream', async () => {
            const buffer = readFileSync('./test/data/countries.fgb');
            const bytes = new Uint8Array(buffer);
            const stream = arrayToStream(bytes.buffer);
            const features = (await takeAsync<FeatureLike>(deserialize(stream))) as Feature[];
            expect(features.length).to.eq(179);
            for (const f of features)
                expect((f.getGeometry() as SimpleGeometry).getCoordinates()?.length).to.be.greaterThan(0);
        });

        it('Bahamas', async () => {
            const expected = {
                type: 'FeatureCollection',
                features: [
                    {
                        type: 'Feature',
                        id: 0,
                        properties: { name: 'The Bahamas' },
                        geometry: {
                            type: 'MultiPolygon',
                            coordinates: [
                                [
                                    [
                                        [-77.53466, 23.75975],
                                        [-77.78, 23.71],
                                        [-78.03405, 24.28615],
                                        [-78.40848, 24.57564],
                                        [-78.19087, 25.2103],
                                        [-77.89, 25.17],
                                        [-77.54, 24.34],
                                        [-77.53466, 23.75975],
                                    ],
                                ],
                                [
                                    [
                                        [-77.82, 26.58],
                                        [-78.91, 26.42],
                                        [-78.98, 26.79],
                                        [-78.51, 26.87],
                                        [-77.85, 26.84],
                                        [-77.82, 26.58],
                                    ],
                                ],
                                [
                                    [
                                        [-77, 26.59],
                                        [-77.17255, 25.87918],
                                        [-77.35641, 26.00735],
                                        [-77.34, 26.53],
                                        [-77.78802, 26.92516],
                                        [-77.79, 27.04],
                                        [-77, 26.59],
                                    ],
                                ],
                            ],
                        },
                    },
                ],
            };
            const actual = (await takeAsync<FeatureLike>(
                deserialize(serialize(geojson.readFeatures(expected))),
            )) as Feature[];
            expect(JSON.parse(g(actual))).to.deep.equal(expected);
        });

        it('Heterogeneous geometry types', async () => {
            const expected = {
                type: 'FeatureCollection',
                features: [
                    {
                        type: 'Feature',
                        id: 0,
                        properties: { name: 'A' },
                        geometry: {
                            type: 'Point',
                            coordinates: [-77.53466, 23.75975],
                        },
                    },
                    {
                        type: 'Feature',
                        id: 1,
                        properties: { name: 'B' },
                        geometry: {
                            type: 'LineString',
                            coordinates: [
                                [-77.53466, 23.75975],
                                [-77, 26.59],
                            ],
                        },
                    },
                ],
            };
            const actual = (await takeAsync<FeatureLike>(
                deserialize(serialize(geojson.readFeatures(expected))),
            )) as Feature[];
            expect(JSON.parse(g(actual))).to.deep.equal(expected);
        });

        it('Heterogeneous geometry types Z', async () => {
            const expected = {
                type: 'FeatureCollection',
                features: [
                    {
                        type: 'Feature',
                        id: 0,
                        properties: { name: 'A' },
                        geometry: {
                            type: 'Point',
                            coordinates: [-77.53466, 23.75975, 12],
                        },
                    },
                    {
                        type: 'Feature',
                        id: 1,
                        properties: { name: 'B' },
                        geometry: {
                            type: 'LineString',
                            coordinates: [
                                [-77.53466, 23.75975, 12],
                                [-77, 26.59, 12],
                            ],
                        },
                    },
                ],
            };
            const actual = (await takeAsync<FeatureLike>(
                deserialize(serialize(geojson.readFeatures(expected))),
            )) as Feature[];
            expect(JSON.parse(g(actual))).to.deep.equal(expected);
        });
    });

    describe('Geometry roundtrips with RenderFeatures', () => {
        it('Point', async () => {
            const expected = makeFeatureCollection('POINT(1.2 -2.1)');
            const s = serialize(expected);
            const actual = (await takeAsync<FeatureLike>(
                deserialize(s, undefined, undefined, undefined, undefined, true),
            )) as RenderFeature[];
            expect(actual[0].getType()).toEqual('Point');
            expect(actual[0].getFlatCoordinates()).toEqual([1.2, -2.1]);
        });

        it('MultiPoint', async () => {
            const expected = makeFeatureCollection('MULTIPOINT(10 40, 40 30, 20 20, 30 10)');
            const actual = (await takeAsync<FeatureLike>(
                deserialize(serialize(expected), undefined, undefined, undefined, undefined, true),
            )) as RenderFeature[];
            expect(actual[0].getType()).toEqual('MultiPoint');
            expect(actual[0].getFlatCoordinates()).toEqual([10, 40, 40, 30, 20, 20, 30, 10]);
        });

        it('LineString', async () => {
            const expected = makeFeatureCollection('LINESTRING(1.2 -2.1, 2.4 -4.8)');
            const actual = (await takeAsync<FeatureLike>(
                deserialize(serialize(expected), undefined, undefined, undefined, undefined, true),
            )) as RenderFeature[];
            expect(actual[0].getType()).toEqual('LineString');
            expect(actual[0].getFlatCoordinates()).toEqual([1.2, -2.1, 2.4, -4.8]);
        });

        it('Polygon', async () => {
            const expected = makeFeatureCollection('POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))');
            const actual = (await takeAsync<FeatureLike>(
                deserialize(serialize(expected), undefined, undefined, undefined, undefined, true),
            )) as RenderFeature[];
            expect(actual[0].getType()).toEqual('Polygon');
            expect(actual[0].getFlatCoordinates()).toEqual([30, 10, 40, 40, 20, 40, 10, 20, 30, 10]);
        });
    });

    describe('Transforms geometry to another projection', () => {
        describe('Point', () => {
            const points = makeFeatureCollection('POINT(13 56)');
            const expected = transform([13, 56], 'EPSG:4326', 'EPSG:3857');

            it('Point to Feature', async () => {
                const s = serialize(points);
                const actual = (await takeAsync<FeatureLike>(
                    deserialize(s, undefined, undefined, undefined, undefined, undefined, 'EPSG:4326', 'EPSG:3857'),
                )) as Feature[];
                const actualGeometry = actual[0].getGeometry() as Point;
                expect(actualGeometry.getFlatCoordinates()).toEqual(expected);
            });

            it('Point to RenderFeature', async () => {
                const s = serialize(points);
                const actual = (await takeAsync<FeatureLike>(
                    deserialize(s, undefined, undefined, undefined, undefined, true, 'EPSG:4326', 'EPSG:3857'),
                )) as RenderFeature[];
                expect(actual[0].getFlatCoordinates()).toEqual(expected);
            });
        });
    });
});
