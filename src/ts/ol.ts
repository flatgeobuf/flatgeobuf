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
import type { DeserializeContext, DeserializeOptions } from './generic/deserialize';
import type { IFeature } from './generic/feature.js';
import {
    deserialize as genericDeserialize,
    deserializeFiltered as genericDeserializeFiltered,
    deserializeStream as genericDeserializeStream,
    serialize as genericSerialize,
} from './generic/featurecollection.js';
import { getFromFeatureFn } from './ol/feature.js';
import type { Rect } from './packedrtree.js';

/**
 * Serialize OpenLayers Features to FlatGeobuf
 * @param features Features to serialize
 */
export function serialize(features: Feature[]): Uint8Array {
    const bytes = genericSerialize(features as IFeature[]);
    return bytes;
}

export interface OlDeserializeOptions extends DeserializeOptions {
    /** Input byte array, stream or URL string */
    input?: Uint8Array | ReadableStream | string;
    /** Filter rectangle */
    rect?: Rect;
    /** Data projection (source). */
    dataProjection?: string;
    /** Features projection (destination). */
    featureProjection?: string;
    /**
     * Feature class to be used when creating features.
     * If performance is the primary concern and features are not going to be
     * modified, consider using RenderFeature (Circle and GeometryCollection are
     * not supported. As coordinates are flattened, multi geometries and polygons
     * with holes are not well rendered).
     */
    renderFeature?: boolean;
}

/**
 * Deserialize FlatGeobuf into OpenLayers Features
 * @param inputOrOptions Input byte array, stream or URL string, or deserializer options including input
 */
export function deserialize(
    inputOrOptions: Uint8Array | ReadableStream | string | OlDeserializeOptions,
): AsyncGenerator<FeatureLike> {
    const actualOptions: OlDeserializeOptions = inputOrOptions instanceof Uint8Array || inputOrOptions instanceof ReadableStream || typeof inputOrOptions === 'string'
        ? { input: inputOrOptions }
        : inputOrOptions;
    const { input, rect: actualRect } = actualOptions;
    const ctx: DeserializeContext = { ...actualOptions, fromFeature: getFromFeatureFn(actualOptions) };
    if (input instanceof Uint8Array)
        return genericDeserialize(ctx) as AsyncGenerator<FeatureLike>;
    if (input instanceof ReadableStream)
        return genericDeserializeStream(ctx) as AsyncGenerator<FeatureLike>;
    if (typeof input === 'string' && actualRect)
        return genericDeserializeFiltered(ctx) as AsyncGenerator<FeatureLike>;
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
 * @param options
 * @returns
 */
export function createLoader(
    source: VectorSource<FeatureLike>,
    url: string,
    strategy: LoadingStrategy = all,
    clear = false,
    options: OlDeserializeOptions = {},
): FeatureLoader<FeatureLike> {
    return async (extent, _resolution, projection, success, failure) => {
        try {
            if (clear) source.clear();
            options.dataProjection = 'EPSG:4326';
            options.featureProjection = projection.getCode();
            const features: FeatureLike[] = [];
            let it: AsyncGenerator<FeatureLike> | undefined;
            if (strategy === all) {
                const response = await fetch(url, { headers: options.headers });
                it = deserialize({ ...options, input: response.body as ReadableStream });
            } else {
                const rect = extentToRect(
                    extent,
                    options.featureProjection,
                    options.dataProjection,
                );
                it = deserialize({ ...options, input: url, rect });
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
 * @param options
 * @returns
 */
export function createTileLoadFunction(
    source: VectorTileSource,
    url: string,
    options: OlDeserializeOptions = {},
) {
    const projection = source.getProjection();
    options.dataProjection = 'EPSG:4326';
    options.featureProjection = projection?.getCode() ?? 'EPSG:3857';
    const tileLoadFunction: LoadFunction = (tile) => {
        const vectorTile = tile as VectorTile<FeatureLike>;
        const loader: FeatureLoader = async (extent) => {
            const rect = extentToRect(
                extent,
                options.featureProjection,
                options.dataProjection,
            );
            const it = deserialize({ ...options, input: url, rect });
            const features: FeatureLike[] = [];
            for await (const feature of it) features.push(feature);
            vectorTile.setFeatures(features);
        };
        vectorTile.setLoader(loader);
    };
    return tileLoadFunction;
}
