import { describe, expect, it } from 'vitest';
import { calcTreeSize, NODE_ITEM_BYTE_LEN } from './packedrtree.js';

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
