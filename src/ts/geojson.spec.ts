import { expect } from 'chai'
import GeoJSONWriter from 'jsts/org/locationtech/jts/io/GeoJSONWriter'
import WKTReader from 'jsts/org/locationtech/jts/io/WKTReader'
import 'mocha'

import { deserialize, serialize } from './geojson'
import { IGeoJsonFeature } from './geojson/feature'

function makeFeatureCollection(wkt: string, properties?: any) {
  return makeFeatureCollectionFromArray([wkt], properties)
}

function makeFeatureCollectionFromArray(wkts: string[], properties?: any) {
  const reader: any = new WKTReader()
  const writer: any = new GeoJSONWriter()
  const geometries = wkts.map(wkt => writer.write(reader.read(wkt)))
  const features = geometries.map(geometry => ({ type: 'Feature', geometry } as IGeoJsonFeature))
  if (properties)
    features.forEach(f => f.properties = properties)
  return {
    type: 'FeatureCollection',
    features,
  }
}

describe('geojson', () => {

  describe('geometry roundtrips', () => {

    it('Point', () => {
      const expected = makeFeatureCollection('POINT(1.2 -2.1)')
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

    it('Points', () => {
      const expected = makeFeatureCollectionFromArray(['POINT(1.2 -2.1)', 'POINT(2.4 -4.8)'])
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

    it('MultiPoint', () => {
      const expected = makeFeatureCollection('MULTIPOINT(10 40, 40 30, 20 20, 30 10)')
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

    it('LineString', () => {
      const expected = makeFeatureCollection('LINESTRING(1.2 -2.1, 2.4 -4.8)')
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

    it('MultiLineString', () => {
      const expected = makeFeatureCollection(`MULTILINESTRING((10 10, 20 20, 10 40),
 (40 40, 30 30, 40 20, 30 10), (50 50, 60 60, 50 90))`)
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

    it('MultiLineStringSinglePart', () => {
      const expected = makeFeatureCollection(`MULTILINESTRING((1.2 -2.1, 2.4 -4.8))`)
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

    it('Polygon', () => {
      const expected = makeFeatureCollection(`POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))`)
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

    it('PolygonWithHole', () => {
      const expected = makeFeatureCollection(`POLYGON ((35 10, 45 45, 15 40, 10 20, 35 10),
 (20 30, 35 35, 30 20, 20 30))`)
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

    it('MultiPolygon', () => {
      const expected = makeFeatureCollection(`MULTIPOLYGON (((30 20, 45 40, 10 40, 30 20)),
 ((15 5, 40 10, 10 20, 5 10, 15 5)))`)
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

    it('MultiPolygonWithHole', () => {
      const expected = makeFeatureCollection(`MULTIPOLYGON (((40 40, 20 45, 45 30, 40 40)),
 ((20 35, 10 30, 10 10, 30 5, 45 20, 20 35), (30 20, 20 15, 20 25, 30 20)))`)
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

    it('MultiPolygonSinglePart', () => {
      const expected = makeFeatureCollection(`MULTIPOLYGON (((30 20, 45 40, 10 40, 30 20)))`)
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

    it('MultiPolygonSinglePartWithHole', () => {
      const expected = makeFeatureCollection(`MULTIPOLYGON (((35 10, 45 45, 15 40, 10 20, 35 10),
 (20 30, 35 35, 30 20, 20 30))))`)
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

  })

  describe('attribute roundtrips', () => {

    it('Number', () => {
      const expected = makeFeatureCollection('POINT(1 1)', {
        test: 1,
      })
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

    it('NumberTwoAttribs', () => {
      const expected = makeFeatureCollection('POINT(1 1)', {
        test: 1,
        test2: 1,
      })
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

    it('NumberWithDecimal', () => {
      const expected = makeFeatureCollection('POINT(1 1)', {
        test: 1.1,
      })
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

    it('Boolean', () => {
      const expected = makeFeatureCollection('POINT(1 1)', {
        test: true,
      })
      const actual = deserialize(serialize(expected))
      expect(actual).to.deep.equal(expected)
    })

  })

})
