import { expect } from 'chai'
import 'mocha'

import { deserialize, serialize } from './ol'
import Feature from 'ol/Feature'
import WKT from 'ol/format/WKT'
import GeoJSON from 'ol/format/GeoJSON'

const format = new WKT()
const geojson = new GeoJSON()

const g = (features: any) => geojson.writeFeatures(features)

function makeFeatureCollection(wkt: string, properties?: any) {
  return makeFeatureCollectionFromArray([wkt], properties)
}

function makeFeatureCollectionFromArray(wkts: string[], properties?: any) {
  const geometries = wkts.map(wkt => format.readGeometry(wkt))
  const features = geometries.map(geometry => {
    const f = new Feature({ geometry })
    return f
  })
  /*if (properties)
    features.forEach(f => f.properties = properties)
  return {
    type: 'FeatureCollection',
    features,
  }*/
  return features
}

describe('ol module', () => {

  describe('Geometry roundtrips', () => {

    it('Point', () => {
      const expected = makeFeatureCollection('POINT(1.2 -2.1)')
      const s = serialize(expected)
      const actual = deserialize(s)
      expect(g(actual)).to.equal(g(expected))
    })

    it('Points', () => {
      const expected = makeFeatureCollectionFromArray(['POINT(1.2 -2.1)', 'POINT(2.4 -4.8)'])
      const actual = deserialize(serialize(expected))
      expect(g(actual)).to.equal(g(expected))
    })

    it('MultiPoint', () => {
      const expected = makeFeatureCollection('MULTIPOINT(10 40, 40 30, 20 20, 30 10)')
      const actual = deserialize(serialize(expected))
      expect(g(actual)).to.equal(g(expected))
    })

    it('LineString', () => {
      const expected = makeFeatureCollection('LINESTRING(1.2 -2.1, 2.4 -4.8)')
      const actual = deserialize(serialize(expected))
      expect(g(actual)).to.equal(g(expected))
    })

    it('MultiLineString', () => {
      const expected = makeFeatureCollection(`MULTILINESTRING((10 10, 20 20, 10 40),
 (40 40, 30 30, 40 20, 30 10), (50 50, 60 60, 50 90))`)
      const actual = deserialize(serialize(expected))
      expect(g(actual)).to.equal(g(expected))
    })

    it('MultiLineStringSinglePart', () => {
      const expected = makeFeatureCollection(`MULTILINESTRING((1.2 -2.1, 2.4 -4.8))`)
      const actual = deserialize(serialize(expected))
      expect(g(actual)).to.equal(g(expected))
    })

    it('Polygon', () => {
      const expected = makeFeatureCollection(`POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))`)
      const actual = deserialize(serialize(expected))
      expect(g(actual)).to.equal(g(expected))
    })

    it('PolygonWithHole', () => {
      const expected = makeFeatureCollection(`POLYGON ((35 10, 45 45, 15 40, 10 20, 35 10),
 (20 30, 35 35, 30 20, 20 30))`)
      const actual = deserialize(serialize(expected))
      expect(g(actual)).to.equal(g(expected))
    })

    it('MultiPolygon', () => {
      const expected = makeFeatureCollection(`MULTIPOLYGON (((30 20, 45 40, 10 40, 30 20)),
 ((15 5, 40 10, 10 20, 5 10, 15 5)))`)
      const actual = deserialize(serialize(expected))
      expect(g(actual)).to.equal(g(expected))
    })

    it('MultiPolygonWithHole', () => {
      const expected = makeFeatureCollection(`MULTIPOLYGON (((40 40, 20 45, 45 30, 40 40)),
 ((20 35, 10 30, 10 10, 30 5, 45 20, 20 35), (30 20, 20 15, 20 25, 30 20)))`)
      const actual = deserialize(serialize(expected))
      expect(g(actual)).to.equal(g(expected))
    })

    it('MultiPolygonSinglePart', () => {
      const expected = makeFeatureCollection(`MULTIPOLYGON (((30 20, 45 40, 10 40, 30 20)))`)
      const actual = deserialize(serialize(expected))
      expect(g(actual)).to.equal(g(expected))
    })

    it('MultiPolygonSinglePartWithHole', () => {
      const expected = makeFeatureCollection(`MULTIPOLYGON (((35 10, 45 45, 15 40, 10 20, 35 10),
 (20 30, 35 35, 30 20, 20 30))))`)
      const actual = deserialize(serialize(expected))
      expect(g(actual)).to.equal(g(expected))
    })

  })

})
