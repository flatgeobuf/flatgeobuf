import Map from 'ol/Map.js';
import OSM from 'ol/source/OSM.js';
import TileLayer from 'ol/layer/Tile.js';
import View from 'ol/View.js';
import VectorLayer from 'ol/layer/Vector.js';
import VectorSource from 'ol/source/Vector.js';
import { bbox } from 'ol/loadingstrategy';

import { createLoader } from '../../src/ts/ol';

const url = '/data/countries.fgb';
const source = new VectorSource({ strategy: bbox });
source.setLoader(createLoader(source, url, 'EPSG:4326', bbox));

new Map({
    layers: [new TileLayer({ source: new OSM() }), new VectorLayer({ source })],
    controls: [],
    target: 'map',
    view: new View({
        center: [1399096, 7494204],
        zoom: 8,
    }),
});
