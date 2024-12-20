import { readFileSync } from 'node:fs';
import path from 'node:path';
import * as url from 'node:url';
import { deserialize } from 'flatgeobuf/lib/mjs/geojson.js';

const __dirname = url.fileURLToPath(new URL('.', import.meta.url));

const data = readFileSync(path.join(__dirname, '../../test/data/countries.fgb'));
const view = new Uint8Array(data.buffer);
const features = deserialize(view);

console.log(features);
