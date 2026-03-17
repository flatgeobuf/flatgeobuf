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
import {
    serialize as genericSerialize,
} from './generic/featurecollection.js';
import type { Rect } from './packedrtree.js';
import { Deserializer, type DeserializerOptions } from "./deserializer";
import type VectorTileSource from "ol/source/VectorTile.js";

export { Deserializer };
export type { DeserializerOptions };

/**
 * Serialize OpenLayers Features to FlatGeobuf
 * @param features Features to serialize
 */
export function serialize(features: Feature[]): Uint8Array {
    const bytes = genericSerialize(features as IFeature[]);
    return bytes;
}

/**
 * Deserialize FlatGeobuf into OpenLayers Features
 * @param deserializer Deserializer instance
 * @param input Input byte array, stream or string
 * @param rect Filter rectangle
 * @param featureProjection Feature projection. Defaults to EPSG:4326
 */
export function deserialize(
    deserializer: Deserializer,
    input: Uint8Array | ReadableStream | string,
    rect?: Rect,
    featureProjection = 'EPSG:4326',
): AsyncGenerator<FeatureLike> {
    if (input instanceof Uint8Array)
        return deserializer.deserialize(input, rect, featureProjection) as AsyncGenerator<FeatureLike>;
    if (input instanceof ReadableStream)
        return deserializer.deserializeStream(input, featureProjection) as AsyncGenerator<FeatureLike>;
    if (typeof input === 'string' && rect)
        return deserializer.deserializeFiltered(input, rect, featureProjection) as AsyncGenerator<FeatureLike>;
    throw new Error('Invalid input type or missing rect for URL input');
}

/**
 * Intended to be used with VectorSource and setLoader to set up
 * a single file FlatGeobuf as a source.
 * @param deserializer
 * @param source
 * @param url
 * @param strategy
 * @param clear
 * @returns
 */
export function createLoader(
    deserializer: Deserializer,
    source: VectorSource<FeatureLike>,
    url: string,
    strategy: LoadingStrategy = all,
    clear = false,
): FeatureLoader<FeatureLike> {
    return async (extent, _resolution, projection, success, failure) => {
        try {
            if (clear) source.clear();
            const features: FeatureLike[] = [];
            let it: AsyncGenerator<FeatureLike> | undefined;
            if (strategy === all) {
                const response = await fetch(url, { headers: deserializer.getHeaders() });
                it = deserialize(
                    deserializer,
                    response.body as ReadableStream,
                    undefined,
                    projection.getCode(),
                );
            } else {
                const rect = deserializer.getRect(extent);
                it = deserialize(deserializer, url, rect, projection.getCode());
            }
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
 * @param deserializer
 * @param source
 * @param url
 * @returns
 */
export function createTileLoadFunction(
    deserializer: Deserializer,
    source: VectorTileSource,
    url: string,
) {
    const projection = source.getProjection();
    const code = projection?.getCode() ?? 'EPSG:3857';
    const tileLoadFunction: LoadFunction = (tile) => {
        const vectorTile = tile as VectorTile<FeatureLike>;
        const loader: FeatureLoader = async (extent) => {
            const rect = deserializer.getRect(extent, code);
            const it = deserialize(deserializer, url, rect, code);
            const features: FeatureLike[] = [];
            for await (const feature of it) features.push(feature);
            vectorTile.setFeatures(features);
        };
        vectorTile.setLoader(loader);
    };
    return tileLoadFunction;
}
