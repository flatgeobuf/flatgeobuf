import { serialize, deserialize as genericDeserialize } from '../generic/featurecollection'
import { IFeature } from '../generic/feature'
import { createGeometry } from './geometry'
import { createFeature } from './feature'

export { serialize as serialize }

export function deserialize(bytes: Uint8Array): IFeature[] {
    return genericDeserialize(bytes, createGeometry, createFeature)
}
