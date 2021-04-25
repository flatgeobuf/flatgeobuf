/* eslint-env browser */
/* global L, flatgeobuf, JSONFormatter */

// basic OSM Leaflet map
let map = L.map('map').setView([41.505, -80.09], 4)
L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
    maxZoom: 19,
    attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
}).addTo(map)

function handleHeaderMeta(headerMeta) {
    const header = document.getElementById('header')
    const formatter = new JSONFormatter(headerMeta, 10)
    header.appendChild(formatter.render())
}

const rect = {
    minX: -102,
    maxX: -100,
    minY: 44,
    maxY: 46,
}

// use flatgeobuf JavaScript API to iterate stream into results (features as geojson)
// NOTE: would be more efficient with a special purpose Leaflet deserializer
let it = flatgeobuf.deserialize('/test/data/UScounties.fgb', rect, handleHeaderMeta)
// handle result
function handleResult(result) {
    if (!result.done) {
        L.geoJSON(result.value).addTo(map)
        it.next().then(handleResult)
    }
}
it.next().then(handleResult)

