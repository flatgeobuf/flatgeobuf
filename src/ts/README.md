# FlatGeobuf for JavaScript / TypeScript

## Building

### Prerequisites

You must have [`pnpm`](https://pnpm.io) installed.

### Install FlatGeobuf dependencies

    pnpm install

### Build

To compile the typescript into a javascript bundle

    pnpm build

See the `scripts` section in [package.json](../../package.json) for other actions.

## Seekable Zstandard

The GeoJSON API can read an externally produced, standard seekable-Zstandard
wrapper from bytes or a range-enabled URL. The wrapper leaves the FlatGeobuf
bytes unchanged. Create wrapped files with external seekable-Zstandard tooling.

```ts
import { geojson } from 'flatgeobuf';

const features = geojson.deserialize('https://example.com/data.fgb.zst', {
    rect: { minX: -106.88, minY: 36.75, maxX: -101.11, maxY: 41.24 },
    seekableZstd: true,
});
```

Indexed reads fetch only the seek-table, FlatGeobuf index, and frames needed for
the requested features. URL sources must support HTTP byte ranges and expose
the `Content-Range` response header for cross-origin requests.
When using a prebuilt browser bundle, serve the generated `zstd.wasm` file next
to the bundle.

### Testing the examples locally.

The examples are hard coded to pull in the publicly released artifact.
If you'd like to test against your local changes, after running `pnpm build`,
update the `<script src=` tags in the examples.

For example:

```diff
diff --git a/examples/leaflet/filtered.html b/examples/leaflet/filtered.html
index 2e13dfc..da4b07e 100644
--- a/examples/leaflet/filtered.html
+++ b/examples/leaflet/filtered.html
@@ -4,7 +4,7 @@
     <link rel="stylesheet" href="/examples/site.css" />
     <script src="https://unpkg.com/underscore@1.13.6/underscore-min.js"></script>
     <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>
-    <script src="https://unpkg.com/flatgeobuf@4.3.3/dist/flatgeobuf-geojson.min.js"></script>
+    <script src="/dist/flatgeobuf-geojson.min.js"></script>
     <script src="https://unpkg.com/json-formatter-js@2.5.23/dist/json-formatter.umd.js"></script>
```

You can start the built in http server with: `pnpm serve`.

Then, open the example in your browser. For example: `open http://localhost:8000/examples/leaflet/filtered.html`.
