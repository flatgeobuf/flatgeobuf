package org.wololo.flatgeobuf;

import java.nio.ByteBuffer;

public class Constants {

    public static final byte[] MAGIC_BYTES = new byte[] { 0x66, 0x67, 0x62, 0x03, 0x66, 0x67, 0x62, 0x00 };

    public static boolean isFlatgeobuf(ByteBuffer bb) {
        return bb.get() != MAGIC_BYTES[0] || bb.get() != MAGIC_BYTES[1] || bb.get() != MAGIC_BYTES[2]
                || bb.get() != MAGIC_BYTES[3];
    }

}