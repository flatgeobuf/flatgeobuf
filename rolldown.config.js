import { defineConfig } from 'rolldown';

export default defineConfig([
    {
        input: './lib/mjs/generic.js',
        output: {
            file: 'dist/flatgeobuf.min.js',
            format: 'umd',
            name: 'flatgeobuf',
            sourcemap: false,
            minify: true,
        },

    },
    {
        input: './lib/mjs/geojson.js',
        output: {
            file: 'dist/flatgeobuf-geojson.min.js',
            format: 'umd',
            name: 'flatgeobuf',
            sourcemap: false,
            minify: true,
        },
    },
    {
        input: './lib/mjs/ol.js',
        external: [
            'ol/Feature.js',
            'ol/format/Feature.js',
            'ol/geom/Point.js',
            'ol/geom/MultiPoint.js',
            'ol/geom/LineString.js',
            'ol/geom/MultiLineString.js',
            'ol/geom/Polygon.js',
            'ol/geom/MultiPolygon.js',
            'ol/geom/GeometryLayout.js',
            'ol/loadingstrategy.js',
            'ol/proj.js',
            'ol/render/Feature.js',
        ],
        output: {
            file: 'dist/flatgeobuf-ol.min.js',
            format: 'umd',
            name: 'flatgeobuf',
            sourcemap: false,
            minify: true,
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
    },
]);
