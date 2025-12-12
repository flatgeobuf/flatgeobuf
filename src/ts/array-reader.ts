import * as flatbuffers from 'flatbuffers';
import { magicbytes, SIZE_PREFIX_LEN } from './constants.js';
import { Feature } from './flat-geobuf/feature.js';
import type { HeaderMeta } from './header-meta.js';
import { fromByteBuffer } from './header-meta.js';
import { calcTreeSize, type Rect, streamSearch } from './packedrtree.js';

interface FeatureWithId {
    id: number;
    feature: Feature;
}

export class ArrayReader {
    private bytes: Uint8Array;
    public header: HeaderMeta;
    private headerLength: number;
    private indexLength: number;

    constructor(bytes: Uint8Array, header: HeaderMeta, headerLength: number, indexLength: number) {
        this.bytes = bytes;
        this.header = header;
        this.headerLength = headerLength;
        this.indexLength = indexLength;
    }

    static open(bytes: Uint8Array): ArrayReader {
        if (!bytes.subarray(0, 3).every((v, i) => magicbytes[i] === v)) {
            throw new Error('Not a FlatGeobuf file');
        }

        const headerLength = new DataView(bytes.buffer).getUint32(magicbytes.length, true);
        const HEADER_MAX_BUFFER_SIZE = 1048576 * 10;
        if (headerLength > HEADER_MAX_BUFFER_SIZE || headerLength < 8) {
            throw new Error('Invalid header size');
        }

        const headerBytes = bytes.subarray(magicbytes.length, magicbytes.length + SIZE_PREFIX_LEN + headerLength);
        const bb = new flatbuffers.ByteBuffer(headerBytes);
        const header = fromByteBuffer(bb);

        const indexLength = calcTreeSize(header.featuresCount, header.indexNodeSize);

        return new ArrayReader(bytes, header, headerLength, indexLength);
    }

    async *selectBbox(rect: Rect): AsyncGenerator<FeatureWithId, void, unknown> {
        const lengthBeforeTree = this.lengthBeforeTree();

        const readNode = async (offsetIntoTree: number, size: number): Promise<ArrayBuffer> => {
            const start = lengthBeforeTree + offsetIntoTree;
            return this.bytes.slice(start, start + size).buffer;
        };

        for await (const searchResult of streamSearch(
            this.header.featuresCount,
            this.header.indexNodeSize,
            rect,
            readNode,
        )) {
            const [featureOffset, featureIdx] = searchResult;
            const feature = this.readFeature(featureOffset);

            yield { id: featureIdx, feature };
        }
    }

    private lengthBeforeTree(): number {
        return magicbytes.length + SIZE_PREFIX_LEN + this.headerLength;
    }

    private lengthBeforeFeatures(): number {
        return this.lengthBeforeTree() + this.indexLength;
    }

    private readFeature(featureOffset: number): Feature {
        const offset = featureOffset + this.lengthBeforeFeatures();

        const featureLength = new DataView(this.bytes.buffer).getUint32(offset, true);
        const featureBytes = this.bytes.subarray(offset + 4, offset + 4 + featureLength);

        const bytesAligned = new Uint8Array(featureLength + SIZE_PREFIX_LEN);
        bytesAligned.set(featureBytes, SIZE_PREFIX_LEN);

        const bb = new flatbuffers.ByteBuffer(bytesAligned);
        bb.setPosition(SIZE_PREFIX_LEN);
        return Feature.getRootAsFeature(bb);
    }
}
