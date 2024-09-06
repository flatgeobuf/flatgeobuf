import Map from 'ol/Map.js';
import OSM from 'ol/source/OSM.js';
import TileLayer from 'ol/layer/Tile.js';
import View from 'ol/View.js';
import VectorTileLayer from 'ol/layer/VectorTile.js';
import VectorTileSource from 'ol/source/VectorTile.js';
import { transformExtent } from 'ol/proj.js';

import { deserialize } from '../../src/ts/ol';
import Feature from 'ol/Feature';
import VectorTile from 'ol/VectorTile';
import { UrlFunction, type LoadFunction } from 'ol/Tile';
import { FeatureLoader } from 'ol/featureloader';

const tileUrlFunction: UrlFunction = (tileCoord) => JSON.stringify(tileCoord);

const tileLoadFunction: LoadFunction = (tile) => {
    const vectorTile = tile as VectorTile<Feature>;
    const loader: FeatureLoader = async (extent) => {
        const [minX, minY, maxX, maxY] = transformExtent(
            extent,
            'EPSG:3857',
            'EPSG:4326',
        );
        const rect = { minX, minY, maxX, maxY };
        const it = deserialize('/data/countries.fgb', rect);
        const features: Feature[] = [];
        for await (const feature of it) features.push(feature);
        features.forEach((f) =>
            f.getGeometry()?.transform('EPSG:4326', 'EPSG:3857'),
        );
        vectorTile.setFeatures(features);
    };
    vectorTile.setLoader(loader);
};

new Map({
    layers: [
        new TileLayer({
            source: new OSM(),
        }),
        new VectorTileLayer({
            source: new VectorTileSource({ tileUrlFunction, tileLoadFunction }),
        }),
    ],
    controls: [],
    target: 'map',
    view: new View({
        center: [1399096, 7494204],
        zoom: 6,
    }),
});
