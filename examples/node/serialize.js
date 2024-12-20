import { readFileSync } from 'node:fs';
import path from 'node:path';
import * as url from 'node:url';
import { deserialize, serialize } from 'flatgeobuf/lib/mjs/geojson.js';

const __dirname = url.fileURLToPath(new URL('.', import.meta.url));

const data = readFileSync(path.join(__dirname, '../../test/data/countries.geojson'));
const jsonData = JSON.parse(data);
// NOTE: be aware that TS/JS implementation creates files with no spatial index
const serializedFeatures = serialize(jsonData);

const reDeserializedData = deserialize(new Uint8Array(serializedFeatures));

console.log(JSON.stringify(reDeserializedData));
