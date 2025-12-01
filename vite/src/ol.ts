import TileLayer from 'ol/layer/Tile.js';
import VectorLayer from 'ol/layer/Vector.js';
import { bbox } from 'ol/loadingstrategy';
import Map from 'ol/Map.js';
// import RenderFeature from "ol/render/Feature.js";
import OSM from 'ol/source/OSM.js';
import VectorSource from 'ol/source/Vector.js';
import View from 'ol/View.js';
import { createLoader } from '../../src/ts/ol';

const url = '/data/countries.fgb';
const source = new VectorSource({ strategy: bbox });
source.setLoader(createLoader(source, url));

new Map({
    layers: [new TileLayer({ source: new OSM() }), new VectorLayer({ source })],
    controls: [],
    target: 'map',
    view: new View({
        center: [1399096, 7494204],
        zoom: 8,
    }),
});
