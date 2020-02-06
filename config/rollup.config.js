import resolve from '@rollup/plugin-node-resolve'
import babel from 'rollup-plugin-babel'
import { terser } from 'rollup-plugin-terser'

const plugins = [
  resolve({
    only: ['flatbuffers', 'slice-source']
  }),
  babel({
    exclude: 'node_modules/**',
    presets: [['@babel/env', {
      modules: false,
      targets: {
        browsers: ['>2%', 'not dead', 'not ie 11']
      }
    }]],
    babelrc: false
  }),
  terser()
]

export default [{
  input: 'lib/geojson.js',
  output: {
    file: 'dist/flatgeobuf-geojson.min.js',
    format: 'umd',
    name: 'flatgeobuf',
    sourcemap: true
  },
  plugins
}, {
  input: 'lib/ol.js',
  output: {
    file: 'dist/flatgeobuf-ol.min.js',
    format: 'umd',
    name: 'flatgeobuf',
    sourcemap: true,
    external: [
      'ol/Feature',
      'ol/geom/Point',
      'ol/geom/MultiPoint',
      'ol/geom/LineString',
      'ol/geom/MultiLineString',
      'ol/geom/Polygon',
      'ol/geom/MultiPolygon',
      'ol/geom/GeometryLayout'
    ],
    globals: {
      'ol/Feature': 'ol.Feature',
      'ol/geom/Point': 'ol.geom.Point',
      'ol/geom/MultiPoint': 'ol.geom.MultiPoint',
      'ol/geom/LineString': 'ol.geom.LineString',
      'ol/geom/MultiLineString': 'ol.geom.MultiLineString',
      'ol/geom/Polygon': 'ol.geom.Polygon',
      'ol/geom/MultiPolygon': 'ol.geom.MultiPolygon',
      'ol/geom/GeometryLayout': 'ol.geom.GeometryLayout'
    }
  },
  plugins
}]