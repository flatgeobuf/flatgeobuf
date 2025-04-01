import type Feature from 'ol/Feature.js';
import type { FeatureLike } from 'ol/Feature.js';
import type { LoadFunction } from 'ol/Tile.js';
import type VectorTile from 'ol/VectorTile.js';
import type { Extent } from 'ol/extent.js';
import type { FeatureLoader } from 'ol/featureloader.js';
import type { Geometry } from 'ol/geom.js';
import { all } from 'ol/loadingstrategy.js';
import { type Projection, transformExtent } from 'ol/proj.js';
import type VectorSource from 'ol/source/Vector.js';
import type { LoadingStrategy } from 'ol/source/Vector.js';
import type VectorTileSource from 'ol/source/VectorTile.js';
import type { TileCoord } from 'ol/tilecoord.js';
import {
    deserialize as fcDeserialize,
    deserializeFiltered as fcDeserializeFiltered,
    deserializeStream as fcDeserializeStream,
    serialize as fcSerialize,
} from './ol/featurecollection.js';

import type { HeaderMetaFn } from './generic.js';
import type { IFeature } from './generic/feature.js';
import type { Rect } from './packedrtree.js';

/**
 * Serialize OpenLayers Features to FlatGeobuf
 * @param features Features to serialize
 */
export function serialize(features: Feature[]): Uint8Array {
    const bytes = fcSerialize(features as IFeature[]);
    return bytes;
}

/**
 * Deserialize FlatGeobuf into OpenLayers Features
 * @param input Input byte array, stream or string
 * @param rect Filter rectangle
 * @param headerMetaFn Callback that will recieve header metadata when available
 */
export function deserialize(
    input: Uint8Array | ReadableStream | string,
    rect?: Rect,
    headerMetaFn?: HeaderMetaFn,
    nocache = false,
): AsyncGenerator<Feature> {
    if (input instanceof Uint8Array) return fcDeserialize(input, rect, headerMetaFn) as AsyncGenerator<Feature>;
    if (input instanceof ReadableStream) return fcDeserializeStream(input, headerMetaFn) as AsyncGenerator<Feature>;
    return fcDeserializeFiltered(input, rect as Rect, headerMetaFn, nocache) as AsyncGenerator<Feature>;
}

async function createIterator(
    url: string,
    srs: string,
    extent: Extent,
    projection: Projection,
    strategy: LoadingStrategy,
) {
    if (strategy === all) {
        const response = await fetch(url);
        return deserialize(response.body as ReadableStream);
    }
    const [minX, minY, maxX, maxY] =
        srs && projection.getCode() !== srs ? transformExtent(extent, projection.getCode(), srs) : extent;
    const rect = { minX, minY, maxX, maxY };
    return deserialize(url, rect);
}

/**
 * Intended to be used with VectorSource and setLoader to setup
 * a single file FlatGeobuf as source.
 * @param source
 * @param url
 * @param srs
 * @param strategy
 * @returns
 */
export function createLoader(
    source: VectorSource,
    url: string,
    srs = 'EPSG:4326',
    strategy: LoadingStrategy = all,
    clear = false,
) {
    const loader: FeatureLoader<Feature> = async (extent, _resolution, projection, success, failure) => {
        try {
            if (clear) source.clear();
            const it = await createIterator(url, srs, extent, projection, strategy);
            const features: Feature<Geometry>[] = [];
            for await (const feature of it) {
                if (srs && projection.getCode() !== srs) feature.getGeometry()?.transform(srs, projection.getCode());
                features.push(feature);
                source.addFeature(feature);
            }
            success?.(features);
        } catch (err) {
            failure?.();
        }
    };
    return loader;
}

/**
 * Intended to be used with VectorTileSource as pseudo URL to key requests.
 * @param tileCoord
 * @returns
 */
export const tileUrlFunction = (tileCoord: TileCoord) => JSON.stringify(tileCoord);

/**
 * Intended to be used with VectorTileSource and setTileLoadFunction to setup
 * a single file FlatGeobuf as source.
 * @param source
 * @param url
 * @param srs
 * @returns
 */
export function createTileLoadFunction(source: VectorTileSource, url: string, srs = 'EPSG:4326') {
    const projection = source.getProjection();
    const code = projection?.getCode() ?? 'EPSG:3857';
    const tileLoadFunction: LoadFunction = (tile) => {
        const vectorTile = tile as VectorTile<FeatureLike>;
        const loader: FeatureLoader = async (extent) => {
            const [minX, minY, maxX, maxY] = srs && code !== srs ? transformExtent(extent, code, srs) : extent;
            const rect = { minX, minY, maxX, maxY };
            const it = deserialize(url, rect);
            const features: Feature[] = [];
            for await (const feature of it) features.push(feature);
            for (const f of features) f.getGeometry()?.transform(srs, code);
            vectorTile.setFeatures(features);
        };
        vectorTile.setLoader(loader);
    };
    return tileLoadFunction;
}
