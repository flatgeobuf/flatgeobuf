import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { defineConfig } from 'rolldown';

const zstdWasmPlugin = {
    name: 'flatgeobuf-zstd-wasm',
    transform(code, id) {
        if (id.includes('/@bokuweb/zstd-wasm/dist/web/index.web.js')) {
            return code
                .replace(
                    'export const init =',
                    'const zstdModuleUrl = (document.currentScript && document.currentScript.src) || location.href;\nexport const init =',
                )
                .replace('import.meta.url', 'zstdModuleUrl');
        }
    },
    generateBundle() {
        this.emitFile({
            type: 'asset',
            fileName: 'zstd.wasm',
            source: readFileSync(resolve('node_modules/@bokuweb/zstd-wasm/dist/web/zstd.wasm')),
        });
    },
};

// Node and bundlers import these through the package's `exports`. Dependencies stay
// external so each environment resolves its own @bokuweb/zstd-wasm variant.
const esmInputs = {
    flatgeobuf: './lib/mjs/flatgeobuf.js',
    generic: './lib/mjs/generic.js',
    geojson: './lib/mjs/geojson.js',
    ol: './lib/mjs/ol.js',
};

export default defineConfig([
    {
        input: esmInputs,
        external: [/^ol\//, 'flatbuffers', '@repeaterjs/repeater', 'slice-source', '@bokuweb/zstd-wasm'],
        output: {
            dir: 'dist/esm',
            format: 'es',
            sourcemap: true,
            minify: { compress: { dropConsole: true }, mangle: false, codegen: { removeWhitespace: false } },
        },
    },
    {
        input: './lib/mjs/generic.js',
        plugins: [zstdWasmPlugin],
        output: {
            file: 'dist/flatgeobuf.min.js',
            format: 'umd',
            name: 'flatgeobuf',
            sourcemap: false,
            minify: { compress: { dropConsole: true } },
        },
    },
    {
        input: './lib/mjs/geojson.js',
        plugins: [zstdWasmPlugin],
        output: {
            file: 'dist/flatgeobuf-geojson.min.js',
            format: 'umd',
            name: 'flatgeobuf',
            sourcemap: false,
            minify: { compress: { dropConsole: true } },
        },
    },
    {
        input: './lib/mjs/ol.js',
        plugins: [zstdWasmPlugin],
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
            minify: { compress: { dropConsole: true } },
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
