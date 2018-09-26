import { expect } from 'chai'
import 'mocha'

import * as api from './api'

const point = {
  coordinates: [1.1, -1.2],
  type: 'Point',
}

const lineString = {
  coordinates: [1.1, -1.2, 2.1, -2.1],
  type: 'LineString',
}

function createFC(geometry: any) {
  return {
    type: 'FeatureCollection',
    features: [{
      type: 'Feature',
      geometry,
    }],
  }
}

function createFCMulti(geometries: any[]) {
  return {
    type: 'FeatureCollection',
    features: geometries.map(geometry => ({
      type: 'Feature',
      geometry,
    })),
  }
}

describe('api', () => {

  describe('roundtrips', () => {

    it('Point roundtrip', () => {
      const fc = createFC(point)
      const data = api.fromGeoJson(fc)
      const geojson = api.toGeoJson(data)
      expect(geojson).to.deep.equal(fc)
    })

    it('Multiple features roundtrip', () => {
      const fc = createFCMulti([point, lineString])
      const data = api.fromGeoJson(fc)
      const geojson = api.toGeoJson(data)
      expect(geojson).to.deep.equal(fc)
    })

    it('LineString roundtrip', () => {
      const fc = createFC(lineString)
      const data = api.fromGeoJson(fc)
      const geojson = api.toGeoJson(data)
      expect(geojson).to.deep.equal(fc)
    })

  })

})
