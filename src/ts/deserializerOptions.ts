import type { HeaderMetaFn } from './generic';

export interface DeserializerOptions {
    /** Data projection (source). */
    dataProjection?: string;
    /** Features projection (destination). */
    featureProjection?: string;
    /** Callback that will receive header metadata when available. */
    headerMetaFn?: HeaderMetaFn;
    /** Header to request the file from. */
    headers?: HeadersInit;
    /** Disable caching of the file. */
    nocache?: boolean;
    /**
     * Feature class to be used when creating features.
     * If performance is the primary concern and features are not going to be
     * modified, consider using RenderFeature (Circle and GeometryCollection are
     * not supported. As coordinates are flattened, multi geometries and polygons
     * with holes are not well rendered).
     */
    renderFeature?: boolean;
}
