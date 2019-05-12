import resolve from 'rollup-plugin-node-resolve'
import babel from 'rollup-plugin-babel'
import { terser } from 'rollup-plugin-terser'

export default {
  input: 'lib/geojson.js',
  output: {
    file: 'dist/flatgeobuf.min.js',
    format: 'umd',
    name: 'flatgeobuf',
    sourcemap: true
  },
  plugins: [
    resolve(),
    babel({
      exclude: 'node_modules/**',
      presets: [['@babel/env', {
        modules: false,
        targets: {
          browsers: ['>1%', 'not dead', 'not ie 11']
        }
      }]],
      babelrc: false
    }),
    terser()
  ]
}