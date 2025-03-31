import * as flatbuffers from "flatbuffers";
import { Feature } from "./flat-geobuf/feature.js";
import { SIZE_PREFIX_LEN, magicbytes } from "./constants.js";
import type { HeaderMeta } from "./header-meta.js";
import { fromByteBuffer } from "./header-meta.js";
import { type Rect, calcTreeSize, search } from "./packedrtree.js";

interface FeatureWithId {
  id: number;
  feature: Feature;
}

export class ArrayReader {
  private bytes: Uint8Array;
  public header: HeaderMeta;
  private headerLength: number;
  private indexLength: number;

  constructor(
    bytes: Uint8Array,
    header: HeaderMeta,
    headerLength: number,
    indexLength: number
  ) {
    this.bytes = bytes;
    this.header = header;
    this.headerLength = headerLength;
    this.indexLength = indexLength;
  }

  static open(bytes: Uint8Array): ArrayReader {
    if (!bytes.subarray(0, 3).every((v, i) => magicbytes[i] === v)) {
      throw new Error("Not a FlatGeobuf file");
    }

    const headerLength = new DataView(bytes.buffer).getUint32(8, true);
    const HEADER_MAX_BUFFER_SIZE = 1048576 * 10;
    if (headerLength > HEADER_MAX_BUFFER_SIZE || headerLength < 8) {
      throw new Error("Invalid header size");
    }

    const headerBytes = bytes.subarray(12, 12 + headerLength);
    const bb = new flatbuffers.ByteBuffer(headerBytes);
    const header = fromByteBuffer(bb);

    const indexLength = calcTreeSize(
      header.featuresCount,
      header.indexNodeSize
    );

    return new ArrayReader(bytes, header, headerLength, indexLength);
  }

  selectBbox(rect: Rect): FeatureWithId[] {
    const lengthBeforeTree = this.lengthBeforeTree();

    const readNode = (
      offsetIntoTree: number,
      size: number
    ): ArrayBuffer => {
      const start = lengthBeforeTree + offsetIntoTree;
      return this.bytes.slice(start, start + size).buffer;
    };
    
    const result: FeatureWithId[] = [];

    for (const searchResult of search(
      this.header.featuresCount,
      this.header.indexNodeSize,
      rect,
      readNode
    )) {
      const [featureOffset, featureIdx] = searchResult;
      const feature = this.readFeature(featureOffset);

      result.push({ id: featureIdx, feature });
    }

    return result
  }

  private lengthBeforeTree(): number {
    return magicbytes.length + SIZE_PREFIX_LEN + this.headerLength;
  }

  private lengthBeforeFeatures(): number {
    return this.lengthBeforeTree() + this.indexLength;
  }

  private readFeature(featureOffset: number): Feature {
    const offset = featureOffset + this.lengthBeforeFeatures();

    const featureLength = new DataView(this.bytes.buffer).getUint32(
      offset,
      true
    );
    const featureBytes = this.bytes.subarray(
      offset + 4,
      offset + 4 + featureLength
    );

    const bytesAligned = new Uint8Array(featureLength + SIZE_PREFIX_LEN);
    bytesAligned.set(featureBytes, SIZE_PREFIX_LEN);

    const bb = new flatbuffers.ByteBuffer(bytesAligned);
    bb.setPosition(SIZE_PREFIX_LEN);
    return Feature.getRootAsFeature(bb);
  }
}
