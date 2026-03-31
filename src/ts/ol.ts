import type { Extent } from 'ol/extent.js';
import type Feature from 'ol/Feature.js';
import type { FeatureLike } from 'ol/Feature.js';
import type { FeatureLoader } from 'ol/featureloader.js';
import { all } from 'ol/loadingstrategy.js';
import { transformExtent } from 'ol/proj.js';
import type VectorSource from 'ol/source/Vector.js';
import type { LoadingStrategy } from 'ol/source/Vector.js';
import type VectorTileSource from 'ol/source/VectorTile.js';
import type { LoadFunction } from 'ol/Tile.js';
import type { TileCoord } from 'ol/tilecoord.js';
import type VectorTile from 'ol/VectorTile.js';
import type { DeserializerOptions } from './deserializerOptions';
import type { IFeature } from './generic/feature.js';
import {
    deserialize as genericDeserialize,
    deserializeFiltered as genericDeserializeFiltered,
    deserializeStream as genericDeserializeStream,
    serialize as genericSerialize,
} from './generic/featurecollection.js';
import { getFromFeatureFn } from './ol/feature.js';
import type { Rect } from './packedrtree.js';

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
 * @param input Input byte array, stream or string
 * @param rect Filter rectangle
 * @param deserializerOptions Deserializer options
 */
export function deserialize(
    input: Uint8Array | ReadableStream | string,
    rect?: Rect,
    deserializerOptions: DeserializerOptions = {},
): AsyncGenerator<FeatureLike> {
    const fromFeature = getFromFeatureFn(
        deserializerOptions.renderFeature ?? false,
        deserializerOptions.dataProjection,
        deserializerOptions.featureProjection,
    );
    if (input instanceof Uint8Array)
        return genericDeserialize(
            input,
            fromFeature,
            rect,
            deserializerOptions.headerMetaFn,
        ) as AsyncGenerator<FeatureLike>;
    if (input instanceof ReadableStream)
        return genericDeserializeStream(
            input,
            fromFeature,
            deserializerOptions.headerMetaFn,
        ) as AsyncGenerator<FeatureLike>;
    if (typeof input === 'string' && rect)
        return genericDeserializeFiltered(
            input,
            rect,
            fromFeature,
            deserializerOptions.headerMetaFn,
            deserializerOptions.nocache,
            deserializerOptions.headers,
        ) as AsyncGenerator<FeatureLike>;
    throw new Error('Invalid input type or missing rect for URL input');
}

function extentToRect(extent: Extent, source?: string, destination?: string): Rect {
    const [minX, minY, maxX, maxY] =
        source && destination && source !== destination ? transformExtent(extent, source, destination) : extent;
    const rect = { minX, minY, maxX, maxY };
    return rect;
}

/**
 * Intended to be used with VectorSource and setLoader to set up
 * a single file FlatGeobuf as a source.
 * @param source
 * @param url
 * @param strategy
 * @param clear
 * @param deserializerOptions
 * @returns
 */
export function createLoader(
    source: VectorSource<FeatureLike>,
    url: string,
    strategy: LoadingStrategy = all,
    clear = false,
    deserializerOptions: DeserializerOptions = {},
): FeatureLoader<FeatureLike> {
    return async (extent, _resolution, projection, success, failure) => {
        try {
            if (clear) source.clear();
            deserializerOptions.dataProjection = 'EPSG:4326';
            deserializerOptions.featureProjection = projection.getCode();
            const features: FeatureLike[] = [];
            let it: AsyncGenerator<FeatureLike> | undefined;
            if (strategy === all) {
                const response = await fetch(url, { headers: deserializerOptions.headers });
                it = deserialize(response.body as ReadableStream, undefined, deserializerOptions);
            } else {
                const rect = extentToRect(
                    extent,
                    deserializerOptions.featureProjection,
                    deserializerOptions.dataProjection,
                );
                it = deserialize(url, rect, deserializerOptions);
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
 * @param source
 * @param url
 * @param deserializerOptions
 * @returns
 */
export function createTileLoadFunction(
    source: VectorTileSource,
    url: string,
    deserializerOptions: DeserializerOptions = {},
) {
    const projection = source.getProjection();
    deserializerOptions.dataProjection = 'EPSG:4326';
    deserializerOptions.featureProjection = projection?.getCode() ?? 'EPSG:3857';
    const tileLoadFunction: LoadFunction = (tile) => {
        const vectorTile = tile as VectorTile<FeatureLike>;
        const loader: FeatureLoader = async (extent) => {
            const rect = extentToRect(
                extent,
                deserializerOptions.featureProjection,
                deserializerOptions.dataProjection,
            );
            const it = deserialize(url, rect, deserializerOptions);
            const features: FeatureLike[] = [];
            for await (const feature of it) features.push(feature);
            vectorTile.setFeatures(features);
        };
        vectorTile.setLoader(loader);
    };
    return tileLoadFunction;
}
