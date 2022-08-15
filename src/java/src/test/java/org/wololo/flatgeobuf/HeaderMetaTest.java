package org.wololo.flatgeobuf;

import com.google.flatbuffers.FlatBufferBuilder;
import org.junit.Before;
import org.junit.Test;
import org.locationtech.jts.geom.Envelope;
import org.wololo.flatgeobuf.generated.GeometryType;

import java.io.*;
import java.util.ArrayList;

import static org.junit.Assert.*;

public class HeaderMetaTest {
    HeaderMeta headerMeta = new HeaderMeta();

    @Before
    public void setUp() throws Exception {
        headerMeta.geometryType = GeometryType.Unknown;
        headerMeta.envelope = new Envelope(0, 1, 0, 1);
        headerMeta.indexNodeSize = 16;
        headerMeta.featuresCount = 16;
        headerMeta.name = "default";
        headerMeta.columns = new ArrayList<>();
    }

    @Test
    public void testWrite() throws IOException {
        File tmpFile = new File("../../test/data/tmpFile20221102.fgb");
        tmpFile.deleteOnExit();
        tmpFile.createNewFile();
        try (FileOutputStream fileOutputStream = new FileOutputStream(tmpFile)) {
            fileOutputStream.write(Constants.MAGIC_BYTES);
            FlatBufferBuilder bufferBuilder = new FlatBufferBuilder();
            HeaderMeta.write(headerMeta, fileOutputStream, bufferBuilder);
        }
        HeaderMeta result;
        try (FileInputStream inputStream = new FileInputStream(tmpFile)) {
            result = HeaderMeta.read(inputStream);
        }
        assertNotNull(result);
        assertEquals(headerMeta.featuresCount, result.featuresCount);
        assertEquals(headerMeta.indexNodeSize, result.indexNodeSize);
        assertEquals(headerMeta.envelope, result.envelope);
        tmpFile.deleteOnExit();
    }
}