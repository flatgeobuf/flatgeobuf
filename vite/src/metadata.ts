import TileLayer from 'ol/layer/Tile.js';
import VectorLayer from 'ol/layer/Vector.js';
import { bbox } from 'ol/loadingstrategy';
import Map from 'ol/Map.js';
// import RenderFeature from "ol/render/Feature.js";
import OSM from 'ol/source/OSM.js';
import VectorSource from 'ol/source/Vector.js';
import View from 'ol/View.js';
import { createLoader } from '../../src/ts/ol';

import { readMetadata } from '../../src/ts/generic/featurecollection';

const url = '/data/UScounties.fgb';

const source = new VectorSource({ strategy: bbox });
source.setLoader(createLoader(source, url));

const map = new Map({
    layers: [
        new TileLayer({ source: new OSM() }),
        new VectorLayer({ source })
    ],
    controls: [],
    target: 'map',
    view: new View({
        projection: 'EPSG:4326',        
        center: [-100, 41],
        zoom: 5,
    }),
});
const mapView = map.getView();

(async () => {  //Set the map viewport to the data dynamically without loading it
    const metadata = await readMetadata(url);
    
    console.log('Read only metadata from remote resource', metadata);
    if(metadata.envelope) {
        const ee = Array.from(metadata.envelope);
        mapView.fit(ee);
    }
})();
