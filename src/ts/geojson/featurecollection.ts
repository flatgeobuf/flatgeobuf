import type {
    FeatureCollection as GeoJsonFeatureCollection,
    GeometryCollection,
    LineString,
    MultiLineString,
    MultiPoint,
    MultiPolygon,
    Point,
    Polygon,
} from 'geojson';
import type { ColumnMeta } from '../column-meta.js';

import { magicbytes } from '../constants.js';
import { buildFeature, type IFeature, type IProperties } from '../generic/feature.js';
import {
    buildHeader,
    deserialize as genericDeserialize,
    deserializeFiltered as genericDeserializeFiltered,
    deserializeStream as genericDeserializeStream,
    mapColumn,
} from '../generic/featurecollection.js';
import { inferGeometryType } from '../generic/header.js';
import type { HeaderMetaFn } from '../generic.js';
import type { HeaderMeta } from '../header-meta.js';
import type { Rect } from '../packedrtree.js';
import { fromFeature } from './feature.js';
import { parseGC, parseGeometry } from './geometry.js';

export function serialize(featurecollection: GeoJsonFeatureCollection, crsCode = 0): Uint8Array {
    const headerMeta = introspectHeaderMeta(featurecollection);
    const header = buildHeader(headerMeta, crsCode);
    const features: Uint8Array[] = featurecollection.features.map((f) =>
        buildFeature(
            f.geometry.type === 'GeometryCollection'
                ? parseGC(f.geometry as GeometryCollection)
                : parseGeometry(
                      f.geometry as Point | MultiPoint | LineString | MultiLineString | Polygon | MultiPolygon,
                  ),
            f.properties as IProperties,
            headerMeta,
        ),
    );
    const featuresLength = features.map((f) => f.length).reduce((a, b) => a + b);
    const uint8 = new Uint8Array(magicbytes.length + header.length + featuresLength);
    uint8.set(header, magicbytes.length);
    let offset = magicbytes.length + header.length;
    for (const feature of features) {
        uint8.set(feature, offset);
        offset += feature.length;
    }
    uint8.set(magicbytes);
    return uint8;
}

export async function* deserialize(
    bytes: Uint8Array,
    rect?: Rect,
    headerMetaFn?: HeaderMetaFn,
): AsyncGenerator<IFeature> {
    yield* genericDeserialize(bytes, fromFeature, rect, headerMetaFn);
}

export function deserializeStream(stream: ReadableStream, headerMetaFn?: HeaderMetaFn): AsyncGenerator<IFeature> {
    return genericDeserializeStream(stream, fromFeature, headerMetaFn);
}

export function deserializeFiltered(
    url: string,
    rect: Rect,
    headerMetaFn?: HeaderMetaFn,
    nocache = false,
    headers: HeadersInit = {},
): AsyncGenerator<IFeature> {
    return genericDeserializeFiltered(url, rect, fromFeature, headerMetaFn, nocache, headers);
}

function introspectHeaderMeta(featurecollection: GeoJsonFeatureCollection): HeaderMeta {
    const feature = featurecollection.features[0];
    const properties = feature.properties;

    let columns: ColumnMeta[] | null = null;
    if (properties) columns = Object.keys(properties).map((k) => mapColumn(properties, k));

    const geometryType = inferGeometryType(featurecollection.features);
    const headerMeta: HeaderMeta = {
        geometryType,
        columns,
        envelope: null,
        featuresCount: featurecollection.features.length,
        indexNodeSize: 0,
        crs: null,
        title: null,
        description: null,
        metadata: null,
    };

    return headerMeta;
}
