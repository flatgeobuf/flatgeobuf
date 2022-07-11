/* eslint-disable no-undef */
import { geojson } from 'flatgeobuf'
import { readFileSync, writeFileSync }  from 'fs'

const expected = {
    type: 'FeatureCollection',
    features: [{
        type: 'Feature',
        geometry: {
            type: 'Point',
            coordinates: [0, 0]
        }
    }]
}

console.log("Input GeoJSON:")
console.log(JSON.stringify(expected, undefined, 1))

const flatgeobuf = geojson.serialize(expected)
console.log(`Serialized input GeoJson into FlatGeobuf (${flatgeobuf.length} bytes)`)

writeFileSync('/tmp/test.fgb', flatgeobuf)
const buffer = readFileSync('/tmp/test.fgb')
const actual = geojson.deserialize(new Uint8Array(buffer))

console.log('FlatGeobuf deserialized back into GeoJSON:')
console.log(JSON.stringify(actual, undefined, 1))