import OlFeature from 'ol/Feature.js';
import Feature from 'ol/Feature';
import { FeatureLoader } from 'ol/featureloader';
import { Projection, transformExtent } from 'ol/proj';
import { Extent } from 'ol/extent';
import VectorSource, { LoadingStrategy } from 'ol/source/Vector.js';
import { all } from 'ol/loadingstrategy';

import { type IFeature } from './generic/feature.js';

import {
    deserialize as fcDeserialize,
    deserializeStream as fcDeserializeStream,
    deserializeFiltered as fcDeserializeFiltered,
    serialize as fcSerialize,
} from './ol/featurecollection.js';
import { type HeaderMetaFn } from './generic.js';
import { type Rect } from './packedrtree.js';

/**
 * Serialize OpenLayers Features to FlatGeobuf
 * @param features Features to serialize
 */
export function serialize(features: OlFeature[]): Uint8Array {
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
    nocache: boolean = false,
): AsyncGenerator<OlFeature> | OlFeature[] {
    if (input instanceof Uint8Array)
        return fcDeserialize(input, headerMetaFn) as OlFeature[];
    else if (input instanceof ReadableStream)
        return fcDeserializeStream(input, headerMetaFn);
    else
        return fcDeserializeFiltered(
            input,
            rect as Rect,
            headerMetaFn,
            nocache,
        );
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
    } else {
        const [minX, minY, maxX, maxY] =
            srs && projection.getCode() !== srs
                ? transformExtent(extent, projection.getCode(), srs)
                : extent;
        const rect = { minX, minY, maxX, maxY };
        return deserialize(url, rect);
    }
}

export function createLoader(
    source: VectorSource,
    url: string,
    srs: string = 'EPSG:4326',
    strategy: LoadingStrategy = all,
) {
    const loader: FeatureLoader<Feature> = async (
        extent,
        _resolution,
        projection,
    ) => {
        const it = await createIterator(url, srs, extent, projection, strategy);
        for await (const feature of it) {
            if (srs && projection.getCode() !== srs)
                feature.getGeometry()?.transform(srs, projection.getCode());
            source.addFeature(feature);
        }
    };
    return loader;
}
