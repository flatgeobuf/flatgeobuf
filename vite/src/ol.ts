import Map from 'ol/Map.js';
import OSM from 'ol/source/OSM.js';
import TileLayer from 'ol/layer/Tile.js';
import View from 'ol/View.js';
import VectorLayer from 'ol/layer/Vector.js';
import VectorSource, { LoadingStrategy } from 'ol/source/Vector.js';
import { all, bbox } from 'ol/loadingstrategy';

import { deserialize } from '../../src/ts/ol';
import Feature from 'ol/Feature';
import { FeatureLoader } from 'ol/featureloader';
import { Projection, transformExtent } from 'ol/proj';
import { Extent } from 'ol/extent';

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

function createLoader(
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
        //source.un('change', source.changed);
        //source.removeFeatures()
        const it = await createIterator(url, srs, extent, projection, strategy);
        for await (const feature of it) {
            if (srs && projection.getCode() !== srs)
                feature.getGeometry()?.transform(srs, projection.getCode());
            source.addFeature(feature);
        }
        //source.changed();
    };
    return loader;
}

const source = new VectorSource({ strategy: bbox });
source.setLoader(
    createLoader(source, '/data/countries.fgb', 'EPSG:4326', bbox),
);

new Map({
    layers: [new TileLayer({ source: new OSM() }), new VectorLayer({ source })],
    controls: [],
    target: 'map',
    view: new View({
        center: [1399096, 7494204],
        zoom: 8,
    }),
});
