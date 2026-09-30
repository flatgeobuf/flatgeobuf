import type { RangeClient } from './http-reader.js';

const SEEK_TABLE_MAGIC = 0x184d2a5e;
const SEEKABLE_MAGIC = 0x8f92eab1;
const SEEK_TABLE_FOOTER_SIZE = 9;
const SEEK_TABLE_HEADER_SIZE = 8;
const SEEK_TABLE_ENTRY_SIZE = 8;
const MAX_UINT32 = 0xffffffff;

interface SeekFrame {
    compressedOffset: number;
    compressedSize: number;
    decompressedOffset: number;
    decompressedSize: number;
}

type ZstdCodec = typeof import('@bokuweb/zstd-wasm');
let codecReady: Promise<ZstdCodec> | undefined;

function loadCodec(wasmUrl?: string): Promise<ZstdCodec> {
    codecReady ??= import('@bokuweb/zstd-wasm').then(async (codec) => {
        await (codec.init as (path?: string) => Promise<void>)(wasmUrl);
        return codec;
    });
    return codecReady;
}

export class SeekableZstdReader implements RangeClient {
    private readonly source: Uint8Array | string;
    private readonly nocache: boolean;
    private readonly headers: HeadersInit;
    private readonly wasmUrl?: string;
    private readonly fileSize: number;
    private readonly frames: SeekFrame[];
    private readonly decompressedFrames = new Map<number, Promise<Uint8Array>>();
    readonly decompressedSize: number;

    private constructor(
        source: Uint8Array | string,
        fileSize: number,
        frames: SeekFrame[],
        decompressedSize: number,
        nocache: boolean,
        headers: HeadersInit,
        wasmUrl?: string,
    ) {
        this.source = source;
        this.fileSize = fileSize;
        this.frames = frames;
        this.decompressedSize = decompressedSize;
        this.nocache = nocache;
        this.headers = headers;
        this.wasmUrl = wasmUrl;
    }

    static async open(
        input: Uint8Array | string,
        nocache = false,
        headers: HeadersInit = {},
        wasmUrl?: string,
    ): Promise<SeekableZstdReader> {
        const tail = await readCompressedSuffix(input, SEEK_TABLE_FOOTER_SIZE, nocache, headers);
        const { fileSize } = tail;
        if (fileSize < SEEK_TABLE_HEADER_SIZE + SEEK_TABLE_FOOTER_SIZE) {
            throw new Error('Invalid seekable Zstandard file: missing seek table');
        }

        const footerOffset = tail.bytes.byteLength - SEEK_TABLE_FOOTER_SIZE;
        const footerView = new DataView(
            tail.bytes.buffer,
            tail.bytes.byteOffset + footerOffset,
            SEEK_TABLE_FOOTER_SIZE,
        );
        if (footerView.getUint32(5, true) !== SEEKABLE_MAGIC) {
            throw new Error('Invalid seekable Zstandard file: missing seekable magic');
        }
        const frameCount = footerView.getUint32(0, true);
        const descriptor = footerView.getUint8(4);
        if (descriptor & 0x7c) throw new Error('Unsupported seek table descriptor');

        const checksumPresent = (descriptor & 0x80) !== 0;
        const entrySize = SEEK_TABLE_ENTRY_SIZE + (checksumPresent ? 4 : 0);
        const tablePayloadSize = frameCount * entrySize + SEEK_TABLE_FOOTER_SIZE;
        if (
            !Number.isSafeInteger(tablePayloadSize) ||
            tablePayloadSize > MAX_UINT32 ||
            tablePayloadSize > fileSize - SEEK_TABLE_HEADER_SIZE
        ) {
            throw new Error('Invalid seekable Zstandard file: seek table is too large');
        }
        const tableOffset = fileSize - SEEK_TABLE_HEADER_SIZE - tablePayloadSize;
        const tableLength = SEEK_TABLE_HEADER_SIZE + tablePayloadSize;
        const tableStart = tableOffset - tail.start;
        const tableFrame =
            tableStart >= 0
                ? tail.bytes.subarray(tableStart)
                : new Uint8Array(
                      await readCompressedRange(input, tableOffset, tableLength, fileSize, nocache, headers),
                  );
        if (tableFrame.byteLength !== tableLength) {
            throw new Error('Invalid seekable Zstandard file: incomplete seek table');
        }
        const tableHeaderView = new DataView(tableFrame.buffer, tableFrame.byteOffset, SEEK_TABLE_HEADER_SIZE);
        if (tableHeaderView.getUint32(0, true) !== SEEK_TABLE_MAGIC) {
            throw new Error('Invalid seekable Zstandard file: invalid seek table frame');
        }
        if (tableHeaderView.getUint32(4, true) !== tablePayloadSize) {
            throw new Error('Invalid seekable Zstandard file: invalid seek table size');
        }

        const table = tableFrame.subarray(SEEK_TABLE_HEADER_SIZE);
        const tableView = new DataView(table.buffer, table.byteOffset, table.byteLength);
        if (
            tableView.getUint32(table.byteLength - SEEK_TABLE_FOOTER_SIZE, true) !== frameCount ||
            tableView.getUint8(table.byteLength - 5) !== descriptor ||
            tableView.getUint32(table.byteLength - 4, true) !== SEEKABLE_MAGIC
        ) {
            throw new Error('Invalid seekable Zstandard file: inconsistent seek table footer');
        }

        const frames: SeekFrame[] = [];
        let compressedOffset = 0;
        let decompressedOffset = 0;
        for (let i = 0; i < frameCount; i++) {
            const offset = i * entrySize;
            const compressedSize = tableView.getUint32(offset, true);
            const decompressedSize = tableView.getUint32(offset + 4, true);
            if (compressedSize === 0) throw new Error('Invalid seekable Zstandard file: empty compressed frame');
            frames.push({
                compressedOffset,
                compressedSize,
                decompressedOffset,
                decompressedSize,
            });
            compressedOffset += compressedSize;
            decompressedOffset += decompressedSize;
            if (!Number.isSafeInteger(compressedOffset) || !Number.isSafeInteger(decompressedOffset)) {
                throw new Error('Invalid seekable Zstandard file: cumulative frame size is too large');
            }
        }
        if (compressedOffset !== tableOffset) {
            throw new Error('Invalid seekable Zstandard file: frame sizes do not match seek table offset');
        }

        return new SeekableZstdReader(input, fileSize, frames, decompressedOffset, nocache, headers, wasmUrl);
    }

    async getRange(begin: number, length: number, _purpose: string): Promise<ArrayBuffer> {
        if (!Number.isSafeInteger(begin) || !Number.isSafeInteger(length) || begin < 0 || length < 0) {
            throw new RangeError('Invalid decompressed byte range');
        }
        if (length === 0 || begin >= this.decompressedSize) return new ArrayBuffer(0);
        const end = Math.min(begin + length, this.decompressedSize);
        if (!Number.isSafeInteger(end)) throw new RangeError('Invalid decompressed byte range');
        const output = new Uint8Array(end - begin);
        let outputOffset = 0;

        for (let i = 0; i < this.frames.length; i++) {
            const frame = this.frames[i];
            const frameEnd = frame.decompressedOffset + frame.decompressedSize;
            if (frame.decompressedSize === 0 || frameEnd <= begin || frame.decompressedOffset >= end) continue;
            const decompressed = await this.loadFrame(i, frame);
            const copyStart = Math.max(begin, frame.decompressedOffset) - frame.decompressedOffset;
            const copyEnd = Math.min(end, frameEnd) - frame.decompressedOffset;
            output.set(decompressed.subarray(copyStart, copyEnd), outputOffset);
            outputOffset += copyEnd - copyStart;
        }

        if (outputOffset !== output.byteLength)
            throw new Error('Invalid seekable Zstandard file: incomplete frame coverage');
        return output.buffer;
    }

    stream(): ReadableStream<Uint8Array> {
        let frameIndex = 0;
        return new ReadableStream<Uint8Array>({
            pull: async (controller) => {
                while (frameIndex < this.frames.length && this.frames[frameIndex].decompressedSize === 0) frameIndex++;
                if (frameIndex >= this.frames.length) {
                    controller.close();
                    return;
                }
                const index = frameIndex++;
                controller.enqueue(await this.loadFrame(index, this.frames[index]));
            },
        });
    }

    private async loadFrame(index: number, frame: SeekFrame): Promise<Uint8Array> {
        let decompressed = this.decompressedFrames.get(index);
        if (!decompressed) {
            decompressed = (async () => {
                const compressed = new Uint8Array(
                    await readCompressedRange(
                        this.source,
                        frame.compressedOffset,
                        frame.compressedSize,
                        this.fileSize,
                        this.nocache,
                        this.headers,
                    ),
                );
                const { decompress } = await loadCodec(this.wasmUrl);
                const bytes = decompress(compressed, { defaultHeapSize: frame.decompressedSize });
                if (bytes.byteLength !== frame.decompressedSize) {
                    throw new Error('Invalid seekable Zstandard file: frame size does not match seek table');
                }
                return bytes;
            })();
            this.decompressedFrames.set(index, decompressed);
        }
        try {
            return await decompressed;
        } catch (error) {
            this.decompressedFrames.delete(index);
            throw error;
        }
    }
}

async function readCompressedSuffix(
    source: Uint8Array | string,
    length: number,
    nocache: boolean,
    headers: HeadersInit,
): Promise<{ bytes: Uint8Array; fileSize: number; start: number }> {
    if (source instanceof Uint8Array) {
        const start = Math.max(source.byteLength - length, 0);
        return { bytes: source.subarray(start), fileSize: source.byteLength, start };
    }

    const requestHeaders = new Headers(headers);
    requestHeaders.set('Range', `bytes=-${length}`);
    if (nocache) requestHeaders.set('Cache-Control', 'no-cache, no-store');
    const response = await fetch(source, { headers: requestHeaders });
    if (response.status !== 206) throw new Error(`Server must support byte ranges (received HTTP ${response.status})`);
    const contentRange = response.headers.get('Content-Range');
    const match = contentRange?.match(/^bytes (\d+)-(\d+)\/(\d+)$/);
    if (!match) throw new Error('Server did not provide a valid Content-Range for the seekable Zstandard file');
    const start = Number(match[1]);
    const end = Number(match[2]);
    const fileSize = Number(match[3]);
    if (
        !Number.isSafeInteger(start) ||
        !Number.isSafeInteger(end) ||
        !Number.isSafeInteger(fileSize) ||
        fileSize < 1 ||
        end !== fileSize - 1 ||
        start !== Math.max(fileSize - length, 0)
    ) {
        throw new Error('Server returned an invalid suffix byte range');
    }
    const bytes = new Uint8Array(await response.arrayBuffer());
    if (bytes.byteLength !== end - start + 1) throw new Error('Server returned an incomplete byte range');
    return { bytes, fileSize, start };
}

async function readCompressedRange(
    source: Uint8Array | string,
    start: number,
    length: number,
    fileSize: number,
    nocache: boolean,
    headers: HeadersInit,
): Promise<ArrayBuffer> {
    if (start < 0 || length < 0 || start + length > fileSize) throw new RangeError('Invalid compressed byte range');
    if (source instanceof Uint8Array) {
        // Buffer.slice returns a view, so copy explicitly to get an ArrayBuffer holding only this range.
        return new Uint8Array(source.subarray(start, start + length)).buffer;
    }

    const requestHeaders = new Headers(headers);
    requestHeaders.set('Range', `bytes=${start}-${start + length - 1}`);
    if (nocache) requestHeaders.set('Cache-Control', 'no-cache, no-store');
    const response = await fetch(source, { headers: requestHeaders });
    if (response.status !== 206) throw new Error(`Server must support byte ranges (received HTTP ${response.status})`);
    const contentRange = response.headers.get('Content-Range');
    if (contentRange !== `bytes ${start}-${start + length - 1}/${fileSize}`) {
        throw new Error('Server returned an unexpected Content-Range');
    }
    const bytes = await response.arrayBuffer();
    if (bytes.byteLength !== length) throw new Error('Server returned an incomplete byte range');
    return bytes;
}
