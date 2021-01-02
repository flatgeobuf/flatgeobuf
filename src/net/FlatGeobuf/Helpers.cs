using System;
using System.IO;
using System.Linq;
using FlatBuffers;
using NetTopologySuite.Geometries;

namespace FlatGeobuf {
    public static class Helpers {
        public static Header ReadHeader(Stream stream)
        {
            var reader = new BinaryReader(stream);
            return ReadHeader(reader, out _);
        }

        public static Header ReadHeader(Stream stream, out int headerSize)
        {
            var reader = new BinaryReader(stream);
            return ReadHeader(reader, out headerSize);
        }

        public static Header ReadHeader(BinaryReader reader)
        {
            return ReadHeader(reader, out _);
        }

        public static Header ReadHeader(BinaryReader reader, out int headerSize)
        {
            var magicBytes = reader.ReadBytes(8);
            if (!magicBytes.SequenceEqual(Constants.MagicBytes))
                throw new Exception("Not a FlatGeobuf file");

            headerSize = reader.ReadInt32();
            var header = Header.GetRootAsHeader(new ByteBuffer(reader.ReadBytes(headerSize)));

            return header;
        }

        public static Envelope GetEnvelope(Header header) 
        {
            if (header.EnvelopeLength == 4)
            {
                var a = header.GetEnvelopeArray();
                return new Envelope(a[0], a[2], a[1], a[3]);
            }
            return null;
        }

        public static int GetCrsCode(Header header)
        {
            return header.Crs?.Code ?? 0;
        }
    }
}