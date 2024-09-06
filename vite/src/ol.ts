import Map from 'ol/Map.js';
import OSM from 'ol/source/OSM.js';
import TileLayer from 'ol/layer/Tile.js';
import View from 'ol/View.js';

new Map({
  layers: [
    new TileLayer({
      source: new OSM(),
    }),
  ],
  controls: [],
  target: 'map',
  view: new View({
    center: [0, 0],
    zoom: 2,
  }),
});