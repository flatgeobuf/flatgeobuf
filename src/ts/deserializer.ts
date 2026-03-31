import type { Extent } from 'ol/extent.js';
import { transformExtent } from 'ol/proj.js';
import type { HeaderMetaFn } from './generic';
import type { IFeature } from './generic/feature';
import {
    type FromFeatureFn,
    deserialize as genericDeserialize,
    deserializeFiltered as genericDeserializeFiltered,
    deserializeStream as genericDeserializeStream,
} from './generic/featurecollection.js';
import { getFromFeatureFn } from './ol/feature.js';
import type { Rect } from './packedrtree.js';

export interface DeserializerOptions {
    /** Data projection. Defaults to EPSG:4326 */
    dataProjection: string;
    /** Callback that will receive header metadata when available. Defaults to undefined. */
    headerMetaFn?: HeaderMetaFn;
    /** Header to request the file from. Defaults to an empty object. */
    headers: HeadersInit;
    /** Disable caching of the file. Defaults to false. */
    nocache: boolean;
    /**
     * Feature class to be used when creating features.
     * If performance is the primary concern and features are not going to be
     * modified, consider using RenderFeature (Circle and GeometryCollection are
     * not supported. As coordinates are flattened, multi geometries and polygons
     * with holes are not well rendered).
     * Default to false (use Feature and not RenderFeature).
     */
    renderFeature: boolean;
}

export class Deserializer {
    private options: DeserializerOptions;

    constructor(options?: Partial<DeserializerOptions>) {
        this.options = {
            renderFeature: false,
            dataProjection: 'EPSG:4326',
            headers: {},
            nocache: false,
            ...options,
        };
    }

    getHeaders(): HeadersInit {
        return this.options.headers;
    }

    setOptions(options: Partial<DeserializerOptions>): void {
        this.options = {
            ...this.options,
            ...options,
        };
    }

    async *deserialize(bytes: Uint8Array, rect?: Rect, featureProjection?: string): AsyncGenerator<IFeature> {
        yield* genericDeserialize(bytes, this.getFromFeatureFn(featureProjection), rect, this.options.headerMetaFn);
    }

    deserializeStream(stream: ReadableStream, featureProjection?: string): AsyncGenerator<IFeature> {
        return genericDeserializeStream(stream, this.getFromFeatureFn(featureProjection), this.options.headerMetaFn);
    }

    deserializeFiltered(url: string, rect: Rect, featureProjection?: string): AsyncGenerator<IFeature> {
        return genericDeserializeFiltered(
            url,
            rect,
            this.getFromFeatureFn(featureProjection),
            this.options.headerMetaFn,
            this.options.nocache,
            this.options.headers,
        );
    }

    getRect(extent: Extent, featureProjection?: string): Rect {
        const [minX, minY, maxX, maxY] =
            featureProjection && this.options.dataProjection !== featureProjection
                ? transformExtent(extent, featureProjection, this.options.dataProjection)
                : extent;
        return { minX, minY, maxX, maxY };
    }

    private getFromFeatureFn(featureProjection?: string): FromFeatureFn {
        return getFromFeatureFn(this.options.renderFeature, this.options.dataProjection, featureProjection);
    }
}
