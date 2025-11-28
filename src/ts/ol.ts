import type { Extent } from 'ol/extent.js';
import Feature from 'ol/Feature.js';
import type { FeatureLike } from 'ol/Feature.js';
import type { FeatureLoader } from 'ol/featureloader.js';
import { all } from 'ol/loadingstrategy.js';
import type VectorSource from 'ol/source/Vector.js';
import type { LoadingStrategy } from 'ol/source/Vector.js';
import type VectorTileSource from 'ol/source/VectorTile.js';
import type { LoadFunction } from 'ol/Tile.js';
import type { TileCoord } from 'ol/tilecoord.js';
import type VectorTile from 'ol/VectorTile.js';
import type { IFeature } from './generic/feature.js';

import type { Rect } from './packedrtree.js';

import {
    deserialize as genericDeserialize,
    deserializeFiltered as genericDeserializeFiltered,
    deserializeStream as genericDeserializeStream,
    serialize as genericSerialize,
} from './generic/featurecollection.js';
import type { HeaderMetaFn } from './generic.js';
import type RenderFeature from 'ol/render/Feature.js';
import { transformExtent, type Projection } from 'ol/proj.js';
import { getFromFeatureFn } from './ol/feature.js';

/**
 * Serialize OpenLayers Features to FlatGeobuf
 * @param fc Features to serialize
 * @param features Features to serialize
 */
export function serialize(features: Feature[]): Uint8Array {
    const bytes = genericSerialize(features as IFeature[]);
    return bytes;
}

/**
 * Deserialize FlatGeobuf into OpenLayers Features
 * @param fc
 * @param input Input byte array, stream or string
 * @param rect Filter rectangle
 */
export function deserialize(
    input: Uint8Array | ReadableStream | string,
    rect?: Rect,
    headerMetaFn?: HeaderMetaFn,
    nocache = false,
    headers: HeadersInit = {},
    featureClass: typeof Feature | typeof RenderFeature = Feature,
    dataProjection: string = 'EPSG:4326',
    featureProjection: string = 'EPSG:4326'
): AsyncGenerator<FeatureLike> {
    const fromFeature = getFromFeatureFn(featureClass, dataProjection, featureProjection)
    if (input instanceof Uint8Array) return genericDeserialize(input, fromFeature, rect, headerMetaFn) as AsyncGenerator<FeatureLike>;
    if (input instanceof ReadableStream) return genericDeserializeStream(input, fromFeature, headerMetaFn) as AsyncGenerator<FeatureLike>;
    if (typeof input === 'string' && rect) return genericDeserializeFiltered(input, rect, fromFeature, headerMetaFn, nocache, headers) as AsyncGenerator<FeatureLike>;
    throw new Error('Invalid input type or missing rect for URL input');
}

function extentToRect(extent: Extent, source?: string, destination?: string): Rect {
    const [minX, minY, maxX, maxY] = source && destination && source !== destination ? transformExtent(extent, source, destination) : extent;
    const rect = { minX, minY, maxX, maxY };
    return rect
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
    source: VectorSource<FeatureLike>,
    url: string,
    srs = 'EPSG:4326',
    strategy: LoadingStrategy = all,
    clear = false,
    headers: HeadersInit = {},
    featureClass: typeof Feature | typeof RenderFeature = Feature
): FeatureLoader<FeatureLike> {
    return async (extent, _resolution, projection, success, failure) => {
        try {
            if (clear) source.clear();
            const features: FeatureLike[] = [];
            let it: AsyncGenerator<FeatureLike> | undefined = undefined
            if (strategy === all) {
                const response = await fetch(url, { headers });
                it = deserialize(response.body as ReadableStream);
            } else {
                const code = projection.getCode()
                const rect = extentToRect(extent, projection.getCode(), srs)
                it = deserialize(url, rect, undefined, false, headers, featureClass, srs, code)
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
 * @param fc
 * @param url
 * @returns
 */
export function createTileLoadFunction(source: VectorTileSource,
    url: string,
    srs = 'EPSG:4326',
    headers: HeadersInit = {},
    featureClass: typeof Feature | typeof RenderFeature = Feature
) {
    const projection = source.getProjection();
    const code = projection?.getCode() ?? 'EPSG:3857';
    const tileLoadFunction: LoadFunction = (tile) => {
        const vectorTile = tile as VectorTile<FeatureLike>;
        const loader: FeatureLoader = async (extent) => {
            const rect = extentToRect(extent, code, srs)
            const it = deserialize(url, rect, undefined, false, headers, featureClass, srs, code);
            const features: FeatureLike[] = [];
            for await (const feature of it) features.push(feature);
            vectorTile.setFeatures(features);
        };
        vectorTile.setLoader(loader);
    };
    return tileLoadFunction;
}
