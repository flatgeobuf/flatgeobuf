import { expect } from 'chai'
import 'mocha'

import * as api from './api'

describe('api', () => {

  it('Point roundtrip', () => {
    const data = api.fromGeoJson({
      coordinates: [0, 0],
      type: 'Point',
    })
    const geojson = api.toGeoJson(data)
    expect(geojson).to.equal(0)
  })

  it('LineString roundtrip', () => {
    const data = api.fromGeoJson({
      coordinates: [0, 0, 0, 0],
      type: 'LineString',
    })
    const geojson = api.toGeoJson(data)
    expect(geojson).to.equal(2)
  })

})
