import type { Extent } from 'ol/extent.js';
import Feature from 'ol/Feature.js';
import { transformExtent } from 'ol/proj.js';
import type RenderFeature from 'ol/render/Feature.js';
import type { IFeature } from '../generic/feature.js';
import {
    type FromFeatureFn,
    deserialize as genericDeserialize,
    deserializeFiltered as genericDeserializeFiltered,
    deserializeStream as genericDeserializeStream,
    serialize as genericSerialize,
} from '../generic/featurecollection.js';
import type { HeaderMetaFn } from '../generic.js';
import type { Rect } from '../packedrtree.js';
import { getFromFeatureFn } from './feature.js';

export interface FeatureCollectionOptions {
    /**
     * Feature class to be used when creating features. The default is Feature.
     * If performance is the primary concern and features are not going to be
     * modified, consider using RenderFeature (Circle and GeometryCollection are
     * not supported. As coordinates are flattened, multi geometries and polygons
     * with holes are not well rendered).
     * Default to ol/Feature.
     */
    featureClass: typeof Feature | typeof RenderFeature;
    /** Data projection. Defaults to EPSG:4326 */
    dataProjection: string;
    /** Header to request the file from. Defaults to an empty object. */
    headers: HeadersInit;
    /** Disable caching of the file. Defaults to false. */
    nocache: boolean;
    /** Feature projection. Defaults to undefined (no conversion). */
    featureProjection?: string;
    /** Callback that will receive header metadata when available. Defaults to undefined. */
    headerMetaFn?: HeaderMetaFn;
}

export class FeatureCollection {
    private options: FeatureCollectionOptions;

    constructor(options?: Partial<FeatureCollectionOptions>) {
        this.options = {
            ...{
                featureClass: Feature,
                dataProjection: 'EPSG:4326',
                headers: {},
                nocache: false,
            },
            ...options,
        };
    }

    getHeaders(): HeadersInit {
        return this.options.headers;
    }

    setOptions(options: Partial<FeatureCollectionOptions>): void {
        this.options = {
            ...this.options,
            ...options,
        };
    }

    getRect(extent: Extent): Rect {
        const [minX, minY, maxX, maxY] =
            this.options.featureProjection && this.options.dataProjection !== this.options.featureProjection
                ? transformExtent(extent, this.options.dataProjection, this.options.featureProjection)
                : extent;
        return { minX, minY, maxX, maxY };
    }

    serialize(features: IFeature[]): Uint8Array {
        return genericSerialize(features);
    }

    async *deserialize(bytes: Uint8Array, rect?: Rect): AsyncGenerator<IFeature> {
        yield* genericDeserialize(bytes, this.getFromFeatureFn(), rect, this.options.headerMetaFn);
    }

    deserializeStream(stream: ReadableStream): AsyncGenerator<IFeature> {
        return genericDeserializeStream(stream, this.getFromFeatureFn(), this.options.headerMetaFn);
    }

    deserializeFiltered(url: string, rect: Rect): AsyncGenerator<IFeature> {
        return genericDeserializeFiltered(
            url,
            rect,
            this.getFromFeatureFn(),
            this.options.headerMetaFn,
            this.options.nocache,
            this.options.headers,
        );
    }

    private getFromFeatureFn(): FromFeatureFn {
        return getFromFeatureFn(this.options.featureClass, this.options.dataProjection, this.options.featureProjection);
    }
}
