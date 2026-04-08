import { Readable } from 'node:stream';

export function arrayToStream(array: ArrayBuffer | Uint8Array): ReadableStream {
    const buffer =
        array instanceof Uint8Array
            ? Buffer.from(array.buffer, array.byteOffset, array.byteLength)
            : Buffer.from(array);
    const nodeStream = Readable.from(buffer);
    return Readable.toWeb(nodeStream) as ReadableStream;
}

export async function takeAsync<T>(asyncIterable: AsyncIterable<T>, count = Number.POSITIVE_INFINITY): Promise<T[]> {
    const result: T[] = [];
    const iterator = asyncIterable[Symbol.asyncIterator]();
    while (result.length < count) {
        const { value, done } = await iterator.next();
        if (done) break;
        result.push(value);
    }
    return result;
}
