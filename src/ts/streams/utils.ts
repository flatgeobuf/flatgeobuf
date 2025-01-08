import { Readable } from 'node:stream';

export function arrayToStream(array: ArrayBuffer): ReadableStream {
    const nodeStream = Readable.from(Buffer.from(array));
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
