import { expect } from 'chai'
import GeoJSONWriter from 'jsts/org/locationtech/jts/io/GeoJSONWriter'
import WKTReader from 'jsts/org/locationtech/jts/io/WKTReader'
import 'mocha'

import * as api from './api'

function makeFeatureCollection(wkt: string) {
  return makeFeatureCollectionFromArray([wkt])
}

function makeFeatureCollectionFromArray(wkts: string[]) {
  const reader: any = new WKTReader()
  const writer: any = new GeoJSONWriter()
  const geometries = wkts.map(wkt => writer.write(reader.read(wkt)))
  const features = geometries.map(geometry => ({ type: 'Feature', geometry }))
  return {
    type: 'FeatureCollection',
    features,
  }
}

describe('api', () => {

  describe('roundtrips', () => {

    it('Point', () => {
      const expected = makeFeatureCollection('POINT(1.2 -2.1)')
      const actual = api.toGeoJson(api.fromGeoJson(expected))
      expect(actual).to.deep.equal(expected)
    })

    it('Points', () => {
      const expected = makeFeatureCollectionFromArray(['POINT(1.2 -2.1)', 'POINT(2.4 -4.8)'])
      const actual = api.toGeoJson(api.fromGeoJson(expected))
      expect(actual).to.deep.equal(expected)
    })

    xit('MultiPoint', () => {
      const expected = makeFeatureCollection('MULTIPOINT(10 40, 40 30, 20 20, 30 10)')
      const actual = api.toGeoJson(api.fromGeoJson(expected))
      expect(actual).to.deep.equal(expected)
    })

    xit('LineString', () => {
      const expected = makeFeatureCollection('LINESTRING(1.2 -2.1, 2.4 -4.8)')
      const actual = api.toGeoJson(api.fromGeoJson(expected))
      expect(actual).to.deep.equal(expected)
    })

  })

})
