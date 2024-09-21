import { readFileSync }  from 'fs'
import path from 'path'
import * as url from 'url'
import { deserialize, serialize } from 'flatgeobuf/lib/mjs/geojson.js'

const __dirname = url.fileURLToPath(new URL('.', import.meta.url))

const data = readFileSync(path.join(__dirname, '../../test/data/countries.geojson'))
const jsonData = JSON.parse(data)
const serializedFeatures = serialize(jsonData)

const reDeserializedData = deserialize(new Uint8Array(serializedFeatures))

console.log(JSON.stringify(reDeserializedData))
