/* eslint-env browser */
/* global ol, proj4, flatgeobuf */

const source = new ol.source.Vector({
    strategy: ol.loadingstrategy.bbox,
    loader: async function (extent) {
        this.clear()
        const rect = { minX: extent[0], minY: extent[1], maxX: extent[2], maxY: extent[3] }
        for await (let feature of flatgeobuf.deserialize('http://flatgeobuf.septima.dk/HNV2021_20210226.fgb', rect)) {
            this.addFeature(feature)
        }
    },
    useSpatialIndex: false
})

const projectionExtent = [120000, 5661139.2, 958860.8, 6500000]
const matrixIds = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '10', '11', '12', '13', '14', '15']
const resolutions = [1638.4, 819.2, 409.6, 204.8, 102.4, 51.2, 25.6, 12.8, 6.4, 3.2, 1.6, .8, .4, .2, .1, .05]
const origin = ol.extent.getTopLeft(projectionExtent)

const tileGrid = new ol.tilegrid.WMTS({
    origin,
    resolutions,
    matrixIds
})

const projection = new ol.proj.Projection({
    code: 'EPSG:25832',
    extent: [120000, 5661139.2, 958860.8, 6500000],
    units: 'm'
})
ol.proj.addProjection(projection)
proj4.defs('EPSG:25832','+proj=utm +zone=32 +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs')
ol.proj.proj4.register(proj4)

new ol.Map({
    layers: [
        new ol.layer.Tile({
            extent: [420000, 6025000, 905000, 6450000],
            source: new ol.source.WMTS({
                url: 'https://services.datafordeler.dk/DKskaermkort/topo_skaermkort_daempet/1.0.0/WMTS?username=RESBNVOCFN&password=uidqARkZKCDw-D3',
                format: 'image/jpeg',
                matrixSet: 'View1',
                style: 'default',
                layer: 'topo_skaermkort_daempet',
                tileGrid
            })
        }),
        new ol.layer.Vector({
            source,
            maxResolution: 4
        })
    ],
    target: 'map',
    view: new ol.View({
        center: [665690, 6237500],
        extent: [420000, 6025000, 905000, 6450000],
        zoom: 1,
        projection: 'EPSG:25832'
    })
})
