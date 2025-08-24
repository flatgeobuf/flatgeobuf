using FlatGeobuf.NTS;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using NetTopologySuite.Geometries;
using System;
using System.IO;
using System.Net.Http;
using System.Threading.Tasks;

namespace FlatGeobuf.Tests.NTS
{
    [TestClass]
    public class AsyncFeatureEnumeratorTest
    {
        [TestMethod]
        public async Task TestCountries()
        {
            var ae = await AsyncFeatureEnumerator.Create(File.OpenRead("../../../../../../test/data/countries.fgb"));
            Assert.IsNotNull(ae);
            Console.WriteLine(ae.Extent.ToString());
            int numFeaturesExpected = ae.NumFeatures;
            int numFeaturesRead = 0;
            while (await ae.MoveNextAsync())
            {
                Console.WriteLine($" {ae.Current.Attributes["id"]} - {ae.Current.Attributes["name"]}");
                numFeaturesRead++;
            }

            Assert.AreEqual(numFeaturesExpected, numFeaturesRead);
        }

        [TestMethod]
        public async Task TestCountriesCount()
        {
            using var fs = File.OpenRead("../../../../../../test/data/countries.fgb");
            var count = await AsyncFeatureEnumerator.CountAsync(fs);
            Assert.AreEqual(179, (int)count);
            Assert.ThrowsException<ObjectDisposedException>(() =>
            {
                fs.Position = 0;
            });
        }

        [TestMethod]
        public async Task TestCountriesWithNoCRS()
        {
            var ae = await AsyncFeatureEnumerator.Create(File.OpenRead("../../../../../../test/data/countries_nocrs.fgb"));
            Assert.IsNotNull(ae);
            Console.WriteLine(ae.Extent.ToString());
            int numFeaturesExpected = ae.NumFeatures;
            int numFeaturesRead = 0;
            while (await ae.MoveNextAsync())
            {
                Console.WriteLine($" {ae.Current.Attributes["id"]} - {ae.Current.Attributes["name"]}");
                numFeaturesRead++;
            }

            Assert.AreEqual(numFeaturesExpected, numFeaturesRead);
        }

        [TestMethod]
        public async Task TestCountriesWithNoCrsCount()
        {
            var count = await AsyncFeatureEnumerator.CountAsync(File.OpenRead("../../../../../../test/data/countries_nocrs.fgb"));
            Assert.AreEqual(179, (int)count);
        }

        [TestMethod]
        public async Task TestCountriesWithNoGeometry()
        {
            var ae = await AsyncFeatureEnumerator.Create(File.OpenRead("../../../../../../test/data/countries_nogeo.fgb"));
            Assert.IsNotNull(ae);
            Console.WriteLine(ae.Extent.ToString());
            int numFeaturesExpected = ae.NumFeatures;
            Assert.AreEqual(0, ae.NumFeatures);

            int numRowsRead = 0;
            while (await ae.MoveNextAsync())
            {
                Console.WriteLine($" {ae.Current.Attributes["id"]} - {ae.Current.Attributes["name"]}");
                numRowsRead++;
            }

            Assert.AreEqual(179, numRowsRead);
        }

        [TestMethod]
        public async Task TestCountriesWithNoGeometryCount()
        {
            var count = await AsyncFeatureEnumerator.CountAsync(File.OpenRead("../../../../../../test/data/countries_nogeo.fgb"));
            Assert.AreEqual(179, (int)count);
        }
        //

        [TestMethod]
        public async Task TestCountriesWithFilter()
        {
            var rect = new Envelope(-16.1, 32.88, 40.18, 84.17);
            var ae = await AsyncFeatureEnumerator.Create(File.OpenRead("../../../../../../test/data/countries.fgb"), rect: rect);
            Assert.IsNotNull(ae);
            Console.WriteLine(ae.Extent.ToString());
            int numFeaturesExpected = ae.NumFeatures;
            int numFeaturesRead = 0;
            while (await ae.MoveNextAsync())
            {
                Console.WriteLine($" {ae.Current.Attributes["id"]} - {ae.Current.Attributes["name"]}");
                numFeaturesRead++;
                Assert.IsTrue(rect.Intersects(ae.Current.Geometry.EnvelopeInternal));
            }

            Assert.IsTrue(numFeaturesExpected > numFeaturesRead);
        }

        [TestMethod]
        public async Task TestCountriesUnseekable()
        {
            var ae = await AsyncFeatureEnumerator.Create(new UnseekableStream(File.OpenRead("../../../../../../test/data/countries.fgb")));
            Assert.IsNotNull(ae);
            Console.WriteLine(ae.Extent.ToString());
            int numFeaturesExpected = ae.NumFeatures;
            int numFeaturesRead = 0;
            while (await ae.MoveNextAsync())
            {
                Console.WriteLine($" {ae.Current.Attributes["id"]} - {ae.Current.Attributes["name"]}");
                numFeaturesRead++;
            }

            Assert.AreEqual(numFeaturesExpected, numFeaturesRead);
        }

        [TestMethod]
        public async Task TestCountriesUnseekableWithFilter()
        {
            var rect = new Envelope(-16.1, 32.88, 40.18, 84.17);
            var ae = await AsyncFeatureEnumerator.Create(new UnseekableStream(File.OpenRead("../../../../../../test/data/countries.fgb")), rect: rect);
            Assert.IsNotNull(ae);
            Console.WriteLine(ae.Extent.ToString());
            int numFeaturesExpected = ae.NumFeatures;
            int numFeaturesRead = 0;
            while (await ae.MoveNextAsync())
            {
                Console.WriteLine($" {ae.Current.Attributes["id"]} - {ae.Current.Attributes["name"]}");
                numFeaturesRead++;
                Assert.IsTrue(rect.Intersects(ae.Current.Geometry.EnvelopeInternal));
            }

            Assert.IsTrue(numFeaturesExpected > numFeaturesRead);
        }

        [TestMethod]
        public async Task TestUSCountiesFromWeb()
        {
            var client = new HttpClient();
            using var rspns = await client.GetAsync("https://flatgeobuf.org/test/data/UScounties.fgb");
            if (rspns == null || !rspns.IsSuccessStatusCode) Assert.Inconclusive("Failed to get USCounties.fgb");

            var strm = await rspns.Content.ReadAsStreamAsync();
            if (strm == null) Assert.Inconclusive("Failed to get USCounties.fgb");

            var ae = await AsyncFeatureEnumerator.Create(strm);
            Assert.IsNotNull(ae);
            int numFeaturesRead = 0;
            while (await ae.MoveNextAsync())
            {
                Console.WriteLine($" {ae.Current.Attributes["FIPS"]} - {ae.Current.Attributes["NAME"]} ({ae.Current.Attributes["STATE"]})");
                numFeaturesRead++;
            }
            Assert.AreEqual(ae.NumFeatures, numFeaturesRead);

        }

        [TestMethod]
        [DataRow([0, true, "ATA"])]
        [DataRow([1, true, "ATF"])]
        [DataRow([5, true, "ZAF"])]
        [DataRow([178, true, "FLK"])]
        [DataRow([179, false, ""])]
        [DataRow([-1, true, "ATA"])]
        [DataRow([1000, false, ""])]
        public async Task TestCountriesWithSkip(int skip, bool expectedCanReadMore, string expectedId)
        {
            var ae = await AsyncFeatureEnumerator.Create(File.OpenRead("../../../../../../test/data/countries.fgb"));
            Assert.IsNotNull(ae);

            await ae.SkipAsync(skip);

            if (!expectedCanReadMore)
            {
                Assert.IsFalse(await ae.MoveNextAsync());
                return;
            }

            Assert.IsTrue(await ae.MoveNextAsync());
            var id = ae.Current.Attributes["id"];
            Assert.AreEqual(expectedId, id);
        }

        [TestMethod]
        [DataRow([0, true, "IRL"])]
        [DataRow([1, true, "GBR"])]
        [DataRow([5, true, "PRT"])]
        [DataRow([40, true, "POL"])]
        [DataRow([-1, true, "IRL"])]
        [DataRow([41, false, ""])]
        [DataRow([1000, false, ""])]
        public async Task TestCountriesWithSkipFilter(int skip, bool expectedCanReadMore, string expectedId)
        {
            var rect = new Envelope(-16.1, 32.88, 40.18, 84.17);
            var ae = await AsyncFeatureEnumerator.Create(File.OpenRead("../../../../../../test/data/countries.fgb"), rect: rect);
            await ae.SkipAsync(skip);

            if (!expectedCanReadMore)
            {
                Assert.IsFalse(await ae.MoveNextAsync());
                return;
            }

            Assert.IsTrue(await ae.MoveNextAsync());
            var id = ae.Current.Attributes["id"];
            Assert.AreEqual(expectedId, id);
        }

        [TestMethod]
        public async Task TestBinary()
        {
            var ae = await AsyncFeatureEnumerator.Create(File.OpenRead("../../../../../../test/data/binary_wkb.fgb"));
            Assert.IsNotNull(ae);

            //One record in file.
            await ae.MoveNextAsync();

            string id;
            byte[] wkb;
            Assert.IsNotNull(id = ae.Current.Attributes["id"] as string);
            Assert.IsNotNull(wkb = ae.Current.Attributes["wkb"] as byte[]);
            Assert.AreEqual("08b2681a1482afff056faced1a3aae40", id);
            Assert.AreEqual(21, wkb.Length);

            //Simple WKB point check
            using var ms = new MemoryStream(wkb);
            using var rdr = new BinaryReader(ms);
            var byteOrder = rdr.Read();
            Assert.AreEqual(0, byteOrder); //Big endian file
            var swapEndian = BitConverter.IsLittleEndian;
            var type = !swapEndian ? rdr.ReadUInt32() : System.Buffers.Binary.BinaryPrimitives.ReverseEndianness(rdr.ReadUInt32());
            Assert.AreEqual(1U, type);
            var x = !swapEndian ? rdr.ReadDouble() : ReverseDouble(rdr.ReadDouble());
            var y = !swapEndian ? rdr.ReadDouble() : ReverseDouble(rdr.ReadDouble());
            Assert.AreEqual(-105.296435079477, x, 1e-12);
            Assert.AreEqual(40.0056839165114, y, 1e-12);

            Assert.IsFalse(await ae.MoveNextAsync());

            double ReverseDouble(double val) => BitConverter.UInt64BitsToDouble(System.Buffers.Binary.BinaryPrimitives.ReverseEndianness(BitConverter.DoubleToUInt64Bits(val)));
        }

        private class UnseekableStream(Stream stream) : Stream
        {
            private readonly Stream _stream = stream;

            public override bool CanRead => _stream.CanRead;

            public override bool CanSeek => false;

            public override bool CanWrite => _stream.CanWrite;

            public override long Length => _stream.Length;

            public override long Position { get => _stream.Position; set => _stream.Position = value; }

            public override void Flush()
            {
                _stream.Flush();
            }

            public override int Read(byte[] buffer, int offset, int count)
            {
                return _stream.Read(buffer, offset, count);
            }

            public override long Seek(long offset, SeekOrigin origin)
            {
                return _stream.Seek(offset, origin);
            }

            public override void SetLength(long value)
            {
                _stream.SetLength(value);
            }

            public override void Write(byte[] buffer, int offset, int count)
            {
                _stream.Write(buffer, offset, count);
            }
        }

    }
}
