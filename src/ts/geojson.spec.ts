import { readFileSync } from 'node:fs';
import GeoJSONWriter from 'jsts/org/locationtech/jts/io/GeoJSONWriter.js';
import WKTReader from 'jsts/org/locationtech/jts/io/WKTReader.js';
import { describe, expect, it } from 'vitest';
import { deserialize, serialize } from './geojson.js';
import type { IGeoJsonFeature } from './geojson/feature.js';
import type { HeaderMeta } from './header-meta.js';
import type { Rect } from './packedrtree.js';
import { arrayToStream, takeAsync } from './streams/utils.js';

import type {
    FeatureCollection as GeoJsonFeatureCollection,
    LineString,
    MultiLineString,
    MultiPoint,
    MultiPolygon,
    Point,
    Polygon,
    Position,
} from 'geojson';
import GeometryFactory from 'jsts/org/locationtech/jts/geom/GeometryFactory.js';

function makeFeatureCollection(
    wkt: string,
    properties?: Record<string, string | number | boolean | object | Uint8Array | undefined>,
) {
    return makeFeatureCollectionFromArray([wkt], properties);
}

function makeFeatureCollectionFromArray(
    wkts: string[],
    properties?: Record<string, string | number | boolean | object | Uint8Array | undefined>,
) {
    const reader = new WKTReader(new GeometryFactory());
    const writer = new GeoJSONWriter();
    const geometries = wkts.map((wkt) => writer.write(reader.read(wkt)));
    const features = geometries.map(
        (geometry, index) =>
            ({
                type: 'Feature',
                id: index,
                geometry,
                properties: {},
            }) as IGeoJsonFeature,
    );
    if (properties) for (const f of features) f.properties = properties;
    return {
        type: 'FeatureCollection',
        features,
    } as GeoJsonFeatureCollection;
}

describe('geojson module', () => {
    describe('Geometry roundtrips', () => {
        it('Point', async () => {
            const expected = makeFeatureCollection('POINT(1.2 -2.1)');
            const s = serialize(expected);
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(s));
            expect(actual).to.deep.equal(expected.features);
        });

        it('Point 3D', async () => {
            const expected = makeFeatureCollection('POINT Z(1.2 -2.1 10)');
            const s = serialize(expected);
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(s));
            expect(actual).to.deep.equal(expected.features);
        });

        it('Point via stream', async () => {
            const expected = makeFeatureCollection('POINT(1.2 -2.1)');
            const s = serialize(expected);
            const stream = arrayToStream(s);
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(stream));
            expect(actual).to.deep.equal(expected.features);
        });

        it('Points', async () => {
            const expected = makeFeatureCollectionFromArray(['POINT(1.2 -2.1)', 'POINT(2.4 -4.8)']);
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('MultiPoint', async () => {
            const expected = makeFeatureCollection('MULTIPOINT(10 40, 40 30, 20 20, 30 10)');
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('LineString', async () => {
            const expected = makeFeatureCollection('LINESTRING(1.2 -2.1, 2.4 -4.8)');
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('LineString 3D', async () => {
            const expected = makeFeatureCollection('LINESTRING Z(1.2 -2.1 1.1, 2.4 -4.8 1.2)');
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('MultiLineString', async () => {
            const expected = makeFeatureCollection(`MULTILINESTRING((10 10, 20 20, 10 40),
 (40 40, 30 30, 40 20, 30 10), (50 50, 60 60, 50 90))`);
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('MultiLineStringSinglePart', async () => {
            const expected = makeFeatureCollection('MULTILINESTRING((1.2 -2.1, 2.4 -4.8))');
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('Polygon', async () => {
            const expected = makeFeatureCollection('POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))');
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('Polygon via stream', async () => {
            const expected = makeFeatureCollection('POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))');
            const s = serialize(expected);
            const stream = arrayToStream(s);
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(stream));
            expect(actual).to.deep.equal(expected.features);
        });

        it('PolygonWithHole', async () => {
            const expected = makeFeatureCollection(`POLYGON ((35 10, 45 45, 15 40, 10 20, 35 10),
 (20 30, 35 35, 30 20, 20 30))`);
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('PolygonWithHole 3D', async () => {
            const expected = makeFeatureCollection(`POLYGON Z((35 10 3, 45 45 4, 15 40 5, 10 20 6, 35 10 7),
 (20 30 3, 35 35 4, 30 20 5, 20 30 6))`);
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('MultiPolygon', async () => {
            const expected = makeFeatureCollection(`MULTIPOLYGON (((30 20, 45 40, 10 40, 30 20)),
 ((15 5, 40 10, 10 20, 5 10, 15 5)))`);
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('MultiPolygonWithHole', async () => {
            const expected = makeFeatureCollection(`MULTIPOLYGON (((40 40, 20 45, 45 30, 40 40)),
 ((20 35, 10 30, 10 10, 30 5, 45 20, 20 35), (30 20, 20 15, 20 25, 30 20)))`);
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            // NOTE: 28 flat coords, ends = [4, 10, 14], endss = [1, 2]
            expect(actual).to.deep.equal(expected.features);
        });

        it('MultiPolygonSinglePart', async () => {
            const expected = makeFeatureCollection('MULTIPOLYGON (((30 20, 45 40, 10 40, 30 20)))');
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('MultiPolygonSinglePartWithHole', async () => {
            const expected = makeFeatureCollection(`MULTIPOLYGON (((35 10, 45 45, 15 40, 10 20, 35 10),
 (20 30, 35 35, 30 20, 20 30))))`);
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            // NOTE: 18 flat coords, ends = [5, 9], endss = null
            expect(actual).to.deep.equal(expected.features);
        });

        it('GeometryCollection', async () => {
            const expected = makeFeatureCollection('GEOMETRYCOLLECTION(POINT(4 6),LINESTRING(4 6,7 10))');
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('GeometryCollection 3D', async () => {
            const expected = makeFeatureCollection('GEOMETRYCOLLECTION Z(POINT Z(4 6 3),LINESTRING Z(4 6 4,7 10 5))');
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('Bahamas', async () => {
            const expected: GeoJsonFeatureCollection = {
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
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('Heterogeneous geometries', async () => {
            const expected = makeFeatureCollectionFromArray([
                'POINT(1.2 -2.1)',
                'LINESTRING(1.2 -2.1, 2.4 -4.8)',
                'MULTIPOLYGON (((30 20, 45 40, 10 40, 30 20)))',
            ]);
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('Long feature properties', async () => {
            const expected: GeoJsonFeatureCollection = {
                type: 'FeatureCollection',
                features: [
                    {
                        type: 'Feature',
                        id: 0,
                        properties: {
                            veryLong1: Array(1024 * 10)
                                .fill('X')
                                .join(''),
                            veryLong2: Array(1024 * 10)
                                .fill('Y')
                                .join(''),
                            veryLong3: Array(1024 * 10)
                                .fill('Z')
                                .join(''),
                        },
                        geometry: {
                            type: 'Point',
                            coordinates: [-77.53466, 23.75975],
                        },
                    },
                ],
            };
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });
    });

    describe('Attribute roundtrips', () => {
        it('Number', async () => {
            const expected = makeFeatureCollection('POINT(1 1)', {
                test: 1,
            });
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('Number with decimals', async () => {
            const expected = makeFeatureCollection('POINT(1 1)', {
                test: 1.1,
            });
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('NumberTwoAttribs', async () => {
            const expected = makeFeatureCollection('POINT(1 1)', {
                test1: 1,
                test2: 1,
            });
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('NumberWithDecimal', async () => {
            const expected = makeFeatureCollection('POINT(1 1)', {
                test: 1.1,
            });
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('Boolean', async () => {
            const expected = makeFeatureCollection('POINT(1 1)', {
                test: true,
            });
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('String', async () => {
            const expected = makeFeatureCollection('POINT(1 1)', {
                test: 'test',
            });
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('Mixed', async () => {
            const expected = makeFeatureCollection('POINT(1 1)', {
                test1: 1,
                test2: 1.1,
                test3: 'test',
                test4: true,
            });
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('Json Value', async () => {
            const expected = makeFeatureCollection('POINT(1 1)', {
                test: { hello: 'world' },
            });
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });

        it('Binary', async () => {
            const expected = makeFeatureCollection('POINT(1 1)', {
                test: new Uint8Array([116, 101, 115, 116]),
            });
            const actual = await takeAsync<IGeoJsonFeature>(deserialize(serialize(expected)));
            expect(actual).to.deep.equal(expected.features);
        });
    });

    describe('Prepared buffers tests', () => {
        it('Should parse countries fgb produced from GDAL byte array', async () => {
            const buffer = readFileSync('./test/data/countries.fgb');
            const bytes = new Uint8Array(buffer);
            let headerMeta: HeaderMeta | undefined;
            const features = await takeAsync<IGeoJsonFeature>(
                deserialize(bytes, undefined, (header: HeaderMeta) => (headerMeta = header)),
            );
            expect(headerMeta?.crs?.code).to.eq(4326);
            expect(features.length).to.eq(179);
            for (const f of features) {
                const g = f.geometry as Point | MultiPoint | LineString | MultiLineString | Polygon | MultiPolygon;
                expect((g.coordinates[0] as Position[]).length).to.be.greaterThan(0);
            }
        });

        it('Should parse countries fgb produced from GDAL stream filter', async () => {
            const r: Rect = { minX: 12, minY: 56, maxX: 12, maxY: 56 };
            const features = await takeAsync<IGeoJsonFeature>(
                deserialize('https://flatgeobuf.septima.dk/countries.fgb', r, undefined, false),
            );
            expect(features.length).to.eq(3);
            for (const f of features)
                expect(((f.geometry as Polygon).coordinates[0] as Position[]).length).to.be.greaterThan(0);
        });

        it('Should parse countries fgb produced from GDAL stream no filter', async () => {
            const buffer = readFileSync('./test/data/countries.fgb');
            const bytes = new Uint8Array(buffer);
            const stream = arrayToStream(bytes.buffer);
            const features = await takeAsync<IGeoJsonFeature>(deserialize(stream));
            expect(features.length).to.eq(179);
            for (const f of features)
                expect(((f.geometry as Polygon).coordinates[0] as Position[]).length).to.be.greaterThan(0);
        });

        it('Should parse UScounties fgb produced from GDAL', async () => {
            const buffer = readFileSync('./test/data/UScounties.fgb');
            const bytes = new Uint8Array(buffer);
            const features = await takeAsync<IGeoJsonFeature>(deserialize(bytes));
            expect(features.length).to.eq(3221);
            for (const f of features) {
                const g = f.geometry as Point | MultiPoint | LineString | MultiLineString | Polygon | MultiPolygon;
                expect((g.coordinates[0] as number[]).length).to.be.greaterThan(0);
            }
        });

        it('Should parse heterogeneous fgb produced from Rust impl', async () => {
            const buffer = readFileSync('./test/data/heterogeneous.fgb');
            const bytes = new Uint8Array(buffer);
            const features = await takeAsync<IGeoJsonFeature>(deserialize(bytes));
            const expected = {
                type: 'FeatureCollection',
                features: [
                    {
                        type: 'Feature',
                        id: 0,
                        properties: {},
                        geometry: { type: 'Point', coordinates: [1.2, -2.1] },
                    },
                    {
                        type: 'Feature',
                        id: 1,
                        properties: {},
                        geometry: {
                            type: 'LineString',
                            coordinates: [
                                [1.2, -2.1],
                                [2.4, -4.8],
                            ],
                        },
                    },
                    {
                        type: 'Feature',
                        id: 2,
                        properties: {},
                        geometry: {
                            type: 'MultiPolygon',
                            coordinates: [
                                [
                                    [
                                        [30, 20],
                                        [45, 40],
                                        [10, 40],
                                        [30, 20],
                                    ],
                                ],
                            ],
                        },
                    },
                ],
            };
            expect(features).to.deep.equal(expected.features);
        });
    });

    describe('Spatial filter', () => {
        it('Should filter by rect when using byte array', async () => {
            const buffer = readFileSync('./test/data/UScounties.fgb');
            const bytes = new Uint8Array(buffer);
            const rect: Rect = {
                minX: -106.88,
                minY: 36.75,
                maxX: -101.11,
                maxY: 41.24,
            };
            const features = await takeAsync<IGeoJsonFeature>(deserialize(bytes, rect));
            expect(features.length).toBe(86);
            const actual = features.slice(0, 4).map((f) => `${f.properties?.NAME}, ${f.properties?.STATE}`);
            const expected = ['Texas, OK', 'Cimarron, OK', 'Taos, NM', 'Colfax, NM'];
            expect(actual).toEqual(expected);
        });

        it('Should filter overlapping multipoly as expected', async () => {
            const buffer = readFileSync('./test/data/mp_overlapping.fgb');
            const bytes = new Uint8Array(buffer);
            const rect: Rect = {
                minX: 14.9,
                minY: 55.1,
                maxX: 14.9,
                maxY: 55.1,
            };
            const features = await takeAsync<IGeoJsonFeature>(deserialize(bytes, rect));
            expect(features.length).toBe(2);
        });
    });
});
