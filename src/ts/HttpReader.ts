import * as flatbuffers from 'flatbuffers'

import { Rect, calcTreeSize, DEFAULT_NODE_SIZE, NODE_ITEM_LEN, streamSearch} from './packedrtree';
import { magicbytes, SIZE_PREFIX_LEN } from './constants';
import Logger from './Logger';
import HeaderMeta from './HeaderMeta';
import { Feature } from './feature';

export class HttpReader {
    private headerClient: BufferedHttpRangeClient;
    private _featureClient?: BufferedHttpRangeClient;
    public header: HeaderMeta;
    private headerLength: number;
    private indexLength: number;

    constructor(headerClient: BufferedHttpRangeClient,
                header: HeaderMeta,
                headerLength: number,
                indexLength: number) {
       this.headerClient = headerClient;
       this.header = header;
       this.headerLength = headerLength;
       this.indexLength = indexLength;
   }

    // Fetch the header, preparing the reader to read Feature data.
    //
    // and potentially some opportunistic fetching of the index.
    static async open(url: string): Promise<HttpReader> {
        // In reality, the header is probably less than half this size, but
        // better to overshoot and fetch an extra kb rather than have to issue
        // a second request.
        const assumedHeaderLength = 2024;

        const headerClient = new BufferedHttpRangeClient(url);

        // Immediately following the header is the optional spatial index, we deliberately fetch
        // a small part of that to skip subsequent requests.
        const assumedIndexLength = (() => {
            // The actual branching factor will be in the header, but since we
            // don't have the header yet, we just guess. The consequence of
            // getting this wrong isn't terminal, it only means we may be
            // fetching slightly more than we need or that we need to make an 
            // extra request later.
            const assumedBranchingFactor = DEFAULT_NODE_SIZE;

            // NOTE: each layer is exponentially larger
            const prefetchedLayers = 3;

            let result = 0;
            let i: number;
            for (i = 0; i < prefetchedLayers; i++) {
                const layer_width = assumedBranchingFactor ** i * NODE_ITEM_LEN;
                result += layer_width;
            }
            return result;
        })();

        const minReqLength = assumedHeaderLength + assumedIndexLength;
        Logger.debug(`fetching header. minReqLength: ${minReqLength} (assumedHeaderLength: ${assumedHeaderLength}, assumedIndexLength: ${assumedIndexLength})`);

        {
            const bytes = new Uint8Array(await headerClient.getRange(0, 8, minReqLength, 'header'));
            if (!bytes.every((v, i) => magicbytes[i] === v)) {
                Logger.error(`bytes: ${bytes} != ${magicbytes}`);
                throw new Error('Not a FlatGeobuf file')
            }
            Logger.debug('magic bytes look good');
        }

        let headerLength: number;
        {
            const bytes = await headerClient.getRange(8, 4, minReqLength, 'header');
            headerLength = new DataView(bytes).getUint32(0, true);
            const HEADER_MAX_BUFFER_SIZE = 1048576 * 10;
            if (headerLength > HEADER_MAX_BUFFER_SIZE || headerLength < 8) {
                // minimum size check avoids panic in FlatBuffers header decoding
                throw new Error('Invalid header size');
            }
            Logger.debug(`headerLength: ${headerLength}`);
        }

        const bytes = await headerClient.getRange(12, headerLength, minReqLength, 'header');
        const bb = new flatbuffers.ByteBuffer(new Uint8Array(bytes));
        const header = HeaderMeta.fromByteBuffer(bb);

        const indexLength = calcTreeSize(header.featuresCount, header.indexNodeSize);

        Logger.debug('completed: opening http reader');
        return new HttpReader(headerClient, header, headerLength, indexLength);
    }

    async* selectBbox(rect: Rect): AsyncGenerator<number[], void, unknown> {
        // Read R-Tree index and build filter for features within bbox
        const lengthBeforeTree = this.lengthBeforeTree();

        const bufferedClient = this.headerClient;
        const readNode = async function(offsetIntoTree: number, size: number): Promise<ArrayBuffer> {
            const minReqLength = 0;
            return bufferedClient.getRange(lengthBeforeTree + offsetIntoTree, size, minReqLength, 'index');
        };

        Logger.debug(`starting: selectBbox, traversing index. lengthBeforeTree: ${lengthBeforeTree}`);
        yield *streamSearch(this.header.featuresCount, this.header.indexNodeSize, rect, readNode);
    }

    lengthBeforeTree(): number {
        // FGB Layout is: [magicbytes (fixed), headerLength (i32), header (variable), Tree (variable), Features (variable)]
        return magicbytes.length + SIZE_PREFIX_LEN + this.headerLength;
    }

    lengthBeforeFeatures(): number {
        return this.lengthBeforeTree() + this.indexLength;
    }

    featureClient(): BufferedHttpRangeClient {
        if (this._featureClient === undefined) {
            this._featureClient = this.headerClient.clone();
        }
        return this._featureClient
    }

    async readFeature(featureOffset: number): Promise<Feature> {
        // read feature data at least 128kb at a time
        const minFeatureReqLength = 128 * 1024;

        const offset = featureOffset + this.lengthBeforeFeatures();

        let featureLength: number;
        {
            const bytes = await this.featureClient().getRange(offset, 
                                                     4,
                                                     minFeatureReqLength,
                                                    'feature length');
            featureLength = new DataView(bytes).getUint32(0, true);
        }
        Logger.debug(`featureOffset: ${offset}, featureLength: ${featureLength}`);

        const byteBuffer = await this.featureClient().getRange(offset + 4,
                                                    featureLength,
                                                    minFeatureReqLength,
                                                   'feature data');
                                                   const bytes = new Uint8Array(byteBuffer);
        const bytesAligned = new Uint8Array(featureLength + SIZE_PREFIX_LEN);
        bytesAligned.set(bytes, SIZE_PREFIX_LEN);
        const bb = new flatbuffers.ByteBuffer(bytesAligned);
        bb.setPosition(SIZE_PREFIX_LEN);
        return Feature.getRootAsFeature(bb)
    }
}

class BufferedHttpRangeClient {
    httpClient: HttpRangeClient;

    private buffer: ArrayBuffer = new ArrayBuffer(0);

    // Byte offset of `buffer` with respect to the beginning of the file being
    // buffered
    private head = 0;

    constructor(source: string | HttpRangeClient) {
        if (typeof source === 'string') {
            this.httpClient = new HttpRangeClient(source); 
        } else {
            this.httpClient = source;
        }
    }

    clone(): BufferedHttpRangeClient {
        const newClient = new BufferedHttpRangeClient(this.httpClient);

        // copy buffer/head to benefit from any already fetched data
        newClient.buffer = this.buffer.slice(0);
        newClient.head = this.head;

        return newClient;
    }

    async getRange(start: number, length: number, minReqLength: number, purpose: string): Promise<ArrayBuffer> {
        Logger.debug(`need Range: ${start}-${start+length-1}`);
        const start_i = start - this.head;
        const end_i = start_i + length;
        if (start_i >= 0 && end_i < this.buffer.byteLength) {
            Logger.debug(`slicing existing Range: ${start_i}-${end_i - 1}`);
            return this.buffer.slice(start_i, end_i);
        }

        const lengthToFetch = Math.max(length, minReqLength);
        this.buffer = await this.httpClient.getRange(start, lengthToFetch, purpose);
        this.head = start;

        return this.buffer.slice(0, length);
    }
}

class HttpRangeClient {
    url: string;
    requestsEverMade = 0;
    bytesEverRequested = 0;

    constructor(url: string) {
        this.url = url;
    }

    async getRange(begin: number, length: number, purpose: string): Promise<ArrayBuffer> {
        this.requestsEverMade += 1;
        this.bytesEverRequested += length;

        const range = `bytes=${begin}-${begin + length - 1}`;
        Logger.debug(`request: #${this.requestsEverMade}, purpose: ${purpose}), bytes: (this_request: ${length}, ever: ${this.bytesEverRequested}), Range: ${range}`);

        const response = await fetch(this.url, {
            headers: {
                'Range': range
            }
        });

        return response.arrayBuffer()
    }
}
