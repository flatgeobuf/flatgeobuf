import { 
    deserialize as deserializeArray,
    deserializeStream,
    deserializeFiltered,
    FromFeatureFn
} from './generic/featurecollection'

import { Rect } from './packedrtree'
import { IFeature } from './generic/feature'
import HeaderMeta from './HeaderMeta'

export type HeaderMetaFn = (headerMeta: HeaderMeta) => never

export function deserialize(
    input: Uint8Array | ReadableStream | string,
    fromFeature: FromFeatureFn,
    rect?: Rect) : any[] | AsyncGenerator<IFeature>
{
    if (input instanceof Uint8Array)
        return deserializeArray(input, fromFeature)
    else if (input instanceof ReadableStream)
        return deserializeStream(input, fromFeature)
    else
        return deserializeFiltered(input, rect, fromFeature)
}

export { 
    serialize
} from './generic/featurecollection'
