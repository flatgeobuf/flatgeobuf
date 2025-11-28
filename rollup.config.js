import babel from '@rollup/plugin-babel';
import resolve from '@rollup/plugin-node-resolve';
import terser from '@rollup/plugin-terser';

const plugins = [
    resolve({
        resolveOnly: ['flatbuffers', 'slice-source', '@repeaterjs/repeater'],
    }),
    babel({
        exclude: 'node_modules/**',
        presets: [
            [
                '@babel/env',
                {
                    modules: false,
                    targets: {
                        browsers: ['>2%', 'not dead', 'not ie 11'],
                    },
                },
            ],
        ],
        babelrc: false,
        babelHelpers: 'bundled',
    }),
    terser(),
];

export default [
    {
        input: './lib/mjs/generic.js',
        output: {
            file: 'dist/flatgeobuf.min.js',
            format: 'umd',
            name: 'flatgeobuf',
            sourcemap: false,
        },
        plugins,
    },
    {
        input: './lib/mjs/geojson.js',
        output: {
            file: 'dist/flatgeobuf-geojson.min.js',
            format: 'umd',
            name: 'flatgeobuf',
            sourcemap: false,
        },
        plugins,
    },
    {
        input: './lib/mjs/ol.js',
        external: [
            'ol/Feature.js',
            'ol/geom/Point.js',
            'ol/geom/MultiPoint.js',
            'ol/geom/LineString.js',
            'ol/geom/MultiLineString.js',
            'ol/geom/Polygon.js',
            'ol/geom/MultiPolygon.js',
            'ol/geom/GeometryLayout.js',
            'ol/loadingstrategy.js',
            'ol/proj.js',
        ],
        output: {
            file: 'dist/flatgeobuf-ol.min.js',
            format: 'umd',
            name: 'flatgeobuf',
            sourcemap: false,
            globals: {
                'ol/Feature.js': 'ol.Feature',
                'ol/format/Feature.js': 'ol.format.Feature',
                'ol/geom/Point.js': 'ol.geom.Point',
                'ol/geom/MultiPoint.js': 'ol.geom.MultiPoint',
                'ol/geom/LineString.js': 'ol.geom.LineString',
                'ol/geom/MultiLineString.js': 'ol.geom.MultiLineString',
                'ol/geom/Polygon.js': 'ol.geom.Polygon',
                'ol/geom/MultiPolygon.js': 'ol.geom.MultiPolygon',
                'ol/geom/GeometryLayout.js': 'ol.geom.GeometryLayout',
                'ol/loadingstrategy.js': 'ol.loadingstrategy',
                'ol/proj.js': 'ol.proj',
                'ol/render/Feature.js': 'ol.render.Feature',
            },
        },
        plugins,
    },
];
