using System;
using System.IO;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace FlatGeobuf.Tests
{
    [TestClass]
    public class HelpersTests
    {
        // Ensures that a truncated stream that only contains the magic bytes plus a
        // partial (or missing) header size prefix is rejected with an error instead
        // of letting BinaryReader silently return fewer bytes than requested, which
        // would previously feed a short/garbage buffer into the FlatBuffers parser.
        [TestMethod]
        public void ReadHeader_TruncatedHeaderSizePrefix_Throws()
        {
            using var stream = new MemoryStream(Constants.MagicBytes);
            using var reader = new BinaryReader(stream);
            Assert.ThrowsException<EndOfStreamException>(() => Helpers.ReadHeader(reader, out _));
        }

        // Ensures that a header size prefix claiming more bytes than are actually
        // present in the stream is rejected with an error instead of returning a
        // header parsed from a truncated buffer.
        [TestMethod]
        public void ReadHeader_HeaderSizeExceedsData_Throws()
        {
            using var stream = new MemoryStream();
            using (var writer = new BinaryWriter(stream, System.Text.Encoding.UTF8, true))
            {
                writer.Write(Constants.MagicBytes);
                writer.Write(1 << 20); // claim a 1 MiB header that isn't actually present
            }
            stream.Position = 0;

            using var reader = new BinaryReader(stream);
            Assert.ThrowsException<InvalidDataException>(() => Helpers.ReadHeader(reader, out _));
        }
    }
}
