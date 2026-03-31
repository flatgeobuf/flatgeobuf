import type { FromFeatureFn } from '../flatgeobuf';
import type { HeaderMetaFn } from '../generic';
import type { Rect } from '../packedrtree';

export interface DeserializeContext extends DeserializeOptions {
    fromFeature: FromFeatureFn;
}

export interface DeserializeOptions {
    /** Input byte array, stream or string URL. */
    input?: Uint8Array | ReadableStream | string;
    /** Filter rectangle for spatial queries. */
    rect?: Rect;
    /** Callback that will receive header metadata when available. */
    headerMetaFn?: HeaderMetaFn;
    /** Header to request the file from. */
    headers?: HeadersInit;
    /** Disable caching of the file. */
    nocache?: boolean;
}
