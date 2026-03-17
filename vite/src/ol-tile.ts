import TileLayer from 'ol/layer/Tile.js';
import VectorTileLayer from 'ol/layer/VectorTile.js';
import Map from 'ol/Map.js';
import OSM from 'ol/source/OSM.js';
import VectorTileSource from 'ol/source/VectorTile.js';
import View from 'ol/View.js';
import { createTileLoadFunction, tileUrlFunction } from '../../src/ts/ol';
import { Deserializer } from "../../src/ts/deserializer";

const url = '/data/countries.fgb';
const source = new VectorTileSource({ tileUrlFunction });
const deserializer = new Deserializer()
source.setTileLoadFunction(createTileLoadFunction(deserializer, source, url));

new Map({
    layers: [new TileLayer({ source: new OSM() }), new VectorTileLayer({ source })],
    controls: [],
    target: 'map',
    view: new View({
        center: [1399096, 7494204],
        zoom: 6,
    }),
});
