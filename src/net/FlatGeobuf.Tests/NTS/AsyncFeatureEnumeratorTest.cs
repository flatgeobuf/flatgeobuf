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

        private class UnseekableStream : Stream
        {
            private readonly Stream _stream;
            public UnseekableStream(Stream stream)

            {
                _stream = stream;
            }

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
