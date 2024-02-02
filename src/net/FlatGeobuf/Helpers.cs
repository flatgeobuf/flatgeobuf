using System;
using System.Buffers;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Google.FlatBuffers;
using NetTopologySuite.Geometries;

namespace FlatGeobuf {
    public static class Helpers {
        public static Header ReadHeader(Stream stream)
        {
            var reader = new BinaryReader(stream, Encoding.UTF8, true);
            return ReadHeader(reader, out _);
        }

        public static Header ReadHeader(Stream stream, out int headerSize)
        {
            var reader = new BinaryReader(stream, Encoding.UTF8, true);
            return ReadHeader(reader, out headerSize);
        }

        public static Header ReadHeader(BinaryReader reader)
        {
            return ReadHeader(reader, out _);
        }

        public static Header ReadHeader(BinaryReader reader, out int headerSize)
        {
            var magicBytes = reader.ReadBytes(8);
            if (!magicBytes.Take(4).SequenceEqual(Constants.MagicBytes.Take(4)))
                throw new Exception("Not a FlatGeobuf file");

            headerSize = reader.ReadInt32();
            var header = Header.GetRootAsHeader(new ByteBuffer(reader.ReadBytes(headerSize)));

            return header;
        }

        /// <summary>
        /// Reads the header of a FlatGeobuf stream
        /// </summary>
        /// <param name="stream">The FlatGeobuf stream</param>
        /// <returns>The header</returns>
        /// <exception cref="InvalidDataException">Thrown if the stream does not contain FlatGeobuf data</exception>
        internal static async ValueTask<HeaderT> ReadHeaderAsync(Stream stream, CancellationToken token)
        {
            byte[] smallBuffer = new byte[8];
            // Read & check magic bytes
            int numRead = await stream.ReadAsync(smallBuffer, 0, 8, token);
            if (numRead != 8) throw new InvalidDataException("Insufficient stream size");
            if (!smallBuffer.Take(4).SequenceEqual(Constants.MagicBytes.Take(4)))
                throw new InvalidDataException("Not a FlatGeobuf stream");
            
            // Read header size
            numRead = await stream.ReadAsync(smallBuffer, 0, 4, token);
            if (numRead != 4) throw new InvalidDataException("Insufficient stream size");
            int headerSize = MemoryMarshal.Read<int>(smallBuffer);

            // Rent a buffer and read header data
            byte[] headerData = ArrayPool<byte>.Shared.Rent(headerSize);
            numRead = await stream.ReadAsync(headerData, 0, headerSize, token);
            if (numRead != headerSize) throw new InvalidDataException("Insufficient stream size");

            // Parse header, return buffer
            var header = Header.GetRootAsHeader(new ByteBuffer(headerData, 0)).UnPack();
            ArrayPool<byte>.Shared.Return(headerData);

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