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
    }
}
