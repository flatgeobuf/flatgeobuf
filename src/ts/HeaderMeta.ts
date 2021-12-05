import * as flatbuffers from 'flatbuffers';

import ColumnMeta from './ColumnMeta.js';
import CrsMeta from './CrsMeta.js';
import { GeometryType } from './flat-geobuf/geometry-type.js';
import { Header } from './flat-geobuf/header.js';

export default class HeaderMeta {
    constructor(
        public geometryType: GeometryType,
        public columns: ColumnMeta[] | null,
        public envelope: Float64Array | null,
        public featuresCount: number,
        public indexNodeSize: number,
        public crs: CrsMeta | null,
        public title: string | null,
        public description: string | null,
        public metadata: string | null
    ) {}

    static fromByteBuffer(bb: flatbuffers.ByteBuffer): HeaderMeta {
        const header = Header.getRootAsHeader(bb);
        const featuresCount = header.featuresCount().toFloat64();
        const indexNodeSize = header.indexNodeSize();

        const columns: ColumnMeta[] = [];
        for (let j = 0; j < header.columnsLength(); j++) {
            const column = header.columns(j);
            if (!column) throw new Error('Column unexpectedly missing');
            if (!column.name())
                throw new Error('Column name unexpectedly missing');
            columns.push(
                new ColumnMeta(
                    column.name() as string,
                    column.type(),
                    column.title(),
                    column.description(),
                    column.width(),
                    column.precision(),
                    column.scale(),
                    column.nullable(),
                    column.unique(),
                    column.primaryKey()
                )
            );
        }
        const crs = header.crs();
        const crsMeta = crs
            ? new CrsMeta(
                  crs.org(),
                  crs.code(),
                  crs.name(),
                  crs.description(),
                  crs.wkt(),
                  crs.codeString()
              )
            : null;
        const headerMeta = new HeaderMeta(
            header.geometryType(),
            columns,
            null,
            featuresCount,
            indexNodeSize,
            crsMeta,
            header.title(),
            header.description(),
            header.metadata()
        );
        return headerMeta;
    }
}
