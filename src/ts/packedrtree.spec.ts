import { readFileSync } from 'node:fs';
import path from 'node:path';
import * as flatbuffers from 'flatbuffers';
import { afterEach, describe, expect, it } from 'vitest';
import Config from './config.js';
import { fromByteBuffer } from './header-meta.js';
import { calcTreeSize, NODE_ITEM_BYTE_LEN, streamSearch } from './packedrtree.js';

describe('calcTreeSize', () => {
    it('Should return the size of the index tree in bytes', () => {
        // A single feature is stored in one leaf node below the root node.
        expect(calcTreeSize(1, 16)).toBe(2 * NODE_ITEM_BYTE_LEN);
        // 179 features: 179 leaf nodes, 12 inner nodes and one root node.
        expect(calcTreeSize(179, 16)).toBe(192 * NODE_ITEM_BYTE_LEN);
    });

    it('Should reject an empty dataset instead of looping forever', () => {
        expect(() => calcTreeSize(0, 16)).toThrow('Number of items must be greater than 0');
        expect(() => calcTreeSize(0, 0)).toThrow('Number of items must be greater than 0');
    });
});

describe('streamSearch', () => {
    const defaultThreshold = Config.global.extraRequestThreshold();
    afterEach(() => Config.global.setExtraRequestThreshold(defaultThreshold));

    const everything = {
        minX: Number.NEGATIVE_INFINITY,
        minY: Number.NEGATIVE_INFINITY,
        maxX: Number.POSITIVE_INFINITY,
        maxY: Number.POSITIVE_INFINITY,
    };

    for (const file of ['mp_overlapping.fgb', 'poly00.fgb', 'countries.fgb']) {
        for (const threshold of [defaultThreshold, 16 * NODE_ITEM_BYTE_LEN, 0]) {
            it(`Should yield every feature once with its length (${file}, threshold ${threshold})`, async () => {
                Config.global.setExtraRequestThreshold(threshold);
                const bytes = new Uint8Array(readFileSync(path.join(__dirname, `../../test/data/${file}`)));
                const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
                const headerLength = view.getUint32(8, true);
                const header = fromByteBuffer(new flatbuffers.ByteBuffer(bytes.subarray(8, 12 + headerLength)));
                const treeStart = 12 + headerLength;
                const featuresStart = treeStart + calcTreeSize(header.featuresCount, header.indexNodeSize);
                const readNode = async (offset: number, length: number) =>
                    bytes.slice(treeStart + offset, treeStart + offset + length).buffer;

                const featureIdxs: number[] = [];
                for await (const [offset, featureIdx, length] of streamSearch(
                    header.featuresCount,
                    header.indexNodeSize,
                    everything,
                    readNode,
                )) {
                    featureIdxs.push(featureIdx);
                    const sizePrefix = view.getUint32(featuresStart + offset, true);
                    if (featureIdx === header.featuresCount - 1) expect(length).toBe(0);
                    else expect(length).toBe(sizePrefix + 4);
                }
                expect(featureIdxs.sort((a, b) => a - b)).toEqual([...Array(header.featuresCount).keys()]);
            });
        }
    }
});
