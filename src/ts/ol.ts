import type { Extent } from 'ol/extent.js';
import type Feature from 'ol/Feature.js';
import type { FeatureLike } from 'ol/Feature.js';
import type { FeatureLoader } from 'ol/featureloader.js';
import { all } from 'ol/loadingstrategy.js';
import type VectorSource from 'ol/source/Vector.js';
import type { LoadingStrategy } from 'ol/source/Vector.js';
import type { LoadFunction } from 'ol/Tile.js';
import type { TileCoord } from 'ol/tilecoord.js';
import type VectorTile from 'ol/VectorTile.js';
import type { IFeature } from './generic/feature.js';

import { FeatureCollection, type FeatureCollectionOptions } from './ol/featurecollection.js';
import type { Rect } from './packedrtree.js';

export { FeatureCollection };
export type { FeatureCollectionOptions };

/**
 * Serialize OpenLayers Features to FlatGeobuf
 * @param fc Features to serialize
 * @param features Features to serialize
 */
export function serialize(fc: FeatureCollection, features: Feature[]): Uint8Array {
    return fc.serialize(features as IFeature[]);
}

/**
 * Deserialize FlatGeobuf into OpenLayers Features
 * @param fc
 * @param input Input byte array, stream or string
 * @param rect Filter rectangle
 */
export function deserialize(
    fc: FeatureCollection,
    input: Uint8Array | ReadableStream | string,
    rect?: Rect,
): AsyncGenerator<FeatureLike> {
    if (input instanceof Uint8Array) return fc.deserialize(input) as AsyncGenerator<FeatureLike>;
    if (input instanceof ReadableStream) return fc.deserializeStream(input) as AsyncGenerator<FeatureLike>;
    return fc.deserializeFiltered(input, rect!) as AsyncGenerator<FeatureLike>;
}

async function createIterator(url: string, extent: Extent, strategy: LoadingStrategy, fc: FeatureCollection) {
    if (strategy === all) {
        const headers = fc.getHeaders();
        const response = await fetch(url, { headers });
        return deserialize(fc, response.body as ReadableStream);
    }
    const rect = fc.getRect(extent);
    return deserialize(fc, url, rect);
}

/**
 * Intended to be used with VectorSource and setLoader to set up
 * a single file FlatGeobuf as a source.
 * @param fc
 * @param source
 * @param url
 * @param strategy
 * @param clear
 * @returns
 */
export function createLoader(
    fc: FeatureCollection,
    source: VectorSource<FeatureLike>,
    url: string,
    strategy: LoadingStrategy = all,
    clear = false,
): FeatureLoader<FeatureLike> {
    return async (extent, _resolution, _projection, success, failure) => {
        try {
            if (clear) source.clear();
            const it = await createIterator(url, extent, strategy, fc);
            const features: FeatureLike[] = [];
            for await (const feature of it) {
                features.push(feature);
                source.addFeature(feature);
            }
            success?.(features);
        } catch (e) {
            console.error(e);
            failure?.();
        }
    };
}

/**
 * Intended to be used with VectorTileSource as pseudo URL to key requests.
 * @param tileCoord
 * @returns
 */
export const tileUrlFunction = (tileCoord: TileCoord) => JSON.stringify(tileCoord);

/**
 * Intended to be used with VectorTileSource and setTileLoadFunction to set up
 * a single file FlatGeobuf as a source.
 * @param fc
 * @param url
 * @returns
 */
export function createTileLoadFunction(fc: FeatureCollection, url: string) {
    const tileLoadFunction: LoadFunction = (tile) => {
        const vectorTile = tile as VectorTile<FeatureLike>;
        const loader: FeatureLoader = async (extent) => {
            const rect = fc.getRect(extent);
            const it = deserialize(fc, url, rect);
            const features: FeatureLike[] = [];
            for await (const feature of it) features.push(feature);
            vectorTile.setFeatures(features);
        };
        vectorTile.setLoader(loader);
    };
    return tileLoadFunction;
}
