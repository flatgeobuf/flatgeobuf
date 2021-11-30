/* eslint-disable no-undef */
import { geojson } from 'flatgeobuf'

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

const actual = geojson.deserialize(flatgeobuf)

console.log('FlatGeobuf deserialized back into GeoJSON:')
console.log(JSON.stringify(actual, undefined, 1))