/* eslint-disable no-undef */
import { geojson } from 'flatgeobuf'

async function streamtest() {
    const response = await fetch('https://flatgeobuf.org/test/data/UScounties.fgb')
    for await (let feature of geojson.deserialize(response.body))
        console.log(JSON.stringify(feature, undefined, 1))
}

streamtest()
