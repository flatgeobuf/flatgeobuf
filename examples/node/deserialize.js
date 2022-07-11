import { readFileSync }  from 'fs'
import path from 'path'
import * as url from 'url'
import { deserialize } from 'flatgeobuf/lib/mjs/geojson.js'

const __dirname = url.fileURLToPath(new URL('.', import.meta.url))

const data = readFileSync(path.join(__dirname, '../../test/data/countries.fgb'))
const view = new Uint8Array(data.buffer)
const features = deserialize(view)

console.log(features)
