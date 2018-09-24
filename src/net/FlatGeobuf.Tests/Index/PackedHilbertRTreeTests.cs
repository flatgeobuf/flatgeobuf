using FlatGeobuf.Index;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace FlatGeobuf.Tests.Index
{
    [TestClass]
    public class PackedHilbertRTreeTests
    {
        [TestMethod]
        public void SingleItemTest()
        {
            var tree = new PackedHilbertRTree(1);
            tree.Add(0, 0, 1, 1);
            tree.Finish();
            var list = tree.Search(0, 0, 1, 1);
            Assert.AreEqual(1, list.Count);
        }

        [TestMethod]
        public void SingleItemRoundtripTest()
        {
            var tree = new PackedHilbertRTree(1);
            tree.Add(0, 0, 1, 1);
            tree.Finish();
            var data = tree.ToBytes();

            var tree2 = new PackedHilbertRTree(1, 16, data);
            var list = tree2.Search(0, 0, 1, 1);

            Assert.AreEqual(1, list.Count);
        }

        [TestMethod]
        public void TwoItemsTest()
        {
            var tree = new PackedHilbertRTree(2);
            tree.Add(0, 0, 1, 1);
            tree.Add(2, 2, 3, 3);
            tree.Finish();
            var result = tree.Search(1, 1, 2, 2);
            Assert.AreEqual(2, result.Count);
        }
    }
}
