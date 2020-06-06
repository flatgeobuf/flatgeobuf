/* eslint-env browser */
/* global ol, flatgeobuf */

const source = new ol.source.Vector({
    loader: async function () {
        const response = await fetch('https://raw.githubusercontent.com/bjornharrtell/flatgeobuf/3.2.1/test/data/UScounties.fgb')
        for await (let feature of flatgeobuf.deserialize(response.body)) {
            feature.getGeometry().transform('EPSG:4326', 'EPSG:3857')
            this.addFeature(feature)
        }
    }
})
new ol.Map({
    layers: [
        new ol.layer.Tile({
            source: new ol.source.OSM()
        }),
        new ol.layer.Vector({
            source
        })
    ],
    target: 'map',
    view: new ol.View({
        center: ol.proj.fromLonLat([-80.09, 41.505]),
        zoom: 4
    })
})
