//! Create and read a [packed Hilbert R-Tree](https://en.wikipedia.org/wiki/Hilbert_R-tree#Packed_Hilbert_R-trees)
//! to enable fast bounding box spatial filtering.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::mem::size_of;
use std::{cmp, f64, u64};

#[allow(non_snake_case)]
#[derive(Clone, PartialEq, Debug)]
/// R-Tree node
pub struct NodeItem {
    minX: f64,   // double
    minY: f64,   // double
    maxX: f64,   // double
    maxY: f64,   // double
    /// Byte offset in feature data section
    offset: u64, // uint64_t
}

#[allow(non_snake_case)]
impl NodeItem {
    pub fn new(minX: f64, minY: f64, maxX: f64, maxY: f64) -> NodeItem {
        NodeItem {
            minX,
            minY,
            maxX,
            maxY,
            offset: 0,
        }
    }

    fn create(offset: u64) -> NodeItem {
        NodeItem {
            minX: f64::INFINITY,
            minY: f64::INFINITY,
            maxX: f64::NEG_INFINITY,
            maxY: f64::NEG_INFINITY,
            offset,
        }
    }

    fn width(&self) -> f64 {
        self.maxX - self.minX
    }

    fn height(&self) -> f64 {
        self.maxY - self.minY
    }

    fn expand(&mut self, r: &NodeItem) {
        if r.minX < self.minX {
            self.minX = r.minX;
        }
        if r.minY < self.minY {
            self.minY = r.minY;
        }
        if r.maxX > self.maxX {
            self.maxX = r.maxX;
        }
        if r.maxY > self.maxY {
            self.maxY = r.maxY;
        }
    }

    pub fn intersects(&self, r: &NodeItem) -> bool {
        if self.maxX < r.minX {
            return false;
        }
        if self.maxY < r.minY {
            return false;
        }
        if self.minX > r.maxX {
            return false;
        }
        if self.minY > r.maxY {
            return false;
        }
        true
    }
}

pub fn calc_extent(nodes: &Vec<NodeItem>) -> NodeItem {
    nodes.iter().fold(NodeItem::create(0), |mut a, b| {
        a.expand(b);
        a
    })
}

#[allow(non_snake_case)]
#[derive(Debug)]
/// Bbox filter search result
pub struct SearchResultItem {
    /// Byte offset in feature data section
    pub offset: usize,
    /// Feature number
    pub index: usize,
}

const HILBERT_MAX: u32 = (1 << 16) - 1;

// Based on public domain code at https://github.com/rawrunprotected/hilbert_curves
#[allow(non_snake_case)]
fn hilbert(x: u32, y: u32) -> u32 {
    let mut a: u32 = x ^ y;
    let mut b: u32 = 0xFFFF ^ a;
    let mut c: u32 = 0xFFFF ^ (x | y);
    let mut d: u32 = x & (y ^ 0xFFFF);

    let mut A: u32 = a | (b >> 1);
    let mut B: u32 = (a >> 1) ^ a;
    let mut C: u32 = ((c >> 1) ^ (b & (d >> 1))) ^ c;
    let mut D: u32 = ((a & (c >> 1)) ^ (d >> 1)) ^ d;

    a = A;
    b = B;
    c = C;
    d = D;
    A = (a & (a >> 2)) ^ (b & (b >> 2));
    B = (a & (b >> 2)) ^ (b & ((a ^ b) >> 2));
    C ^= (a & (c >> 2)) ^ (b & (d >> 2));
    D ^= (b & (c >> 2)) ^ ((a ^ b) & (d >> 2));

    a = A;
    b = B;
    c = C;
    d = D;
    A = (a & (a >> 4)) ^ (b & (b >> 4));
    B = (a & (b >> 4)) ^ (b & ((a ^ b) >> 4));
    C ^= (a & (c >> 4)) ^ (b & (d >> 4));
    D ^= (b & (c >> 4)) ^ ((a ^ b) & (d >> 4));

    a = A;
    b = B;
    c = C;
    d = D;
    C ^= (a & (c >> 8)) ^ (b & (d >> 8));
    D ^= (b & (c >> 8)) ^ ((a ^ b) & (d >> 8));

    a = C ^ (C >> 1);
    b = D ^ (D >> 1);

    let mut i0: u32 = x ^ y;
    let mut i1: u32 = b | (0xFFFF ^ (i0 | a));

    i0 = (i0 | (i0 << 8)) & 0x00FF00FF;
    i0 = (i0 | (i0 << 4)) & 0x0F0F0F0F;
    i0 = (i0 | (i0 << 2)) & 0x33333333;
    i0 = (i0 | (i0 << 1)) & 0x55555555;

    i1 = (i1 | (i1 << 8)) & 0x00FF00FF;
    i1 = (i1 | (i1 << 4)) & 0x0F0F0F0F;
    i1 = (i1 | (i1 << 2)) & 0x33333333;
    i1 = (i1 | (i1 << 1)) & 0x55555555;

    let value: u32 = (i1 << 1) | i0;

    value
}

#[allow(non_snake_case)]
fn hilbert_bbox(r: &NodeItem, hilbertMax: u32, extent: &NodeItem) -> u32 {
    // calculate bbox center and scale to hilbertMax
    // Hint from @vmx: Why not OMT tree (http://ceur-ws.org/Vol-74/files/FORUM_18.pdf)?
    let x = (hilbertMax as f64 * ((r.minX + r.maxX) / 2.0 - extent.minX) / extent.width()).floor()
        as u32;
    let y = (hilbertMax as f64 * ((r.minY + r.maxY) / 2.0 - extent.minY) / extent.height()).floor()
        as u32;
    hilbert(x, y)
}

pub fn hilbert_sort(items: &mut Vec<NodeItem>) {
    let extent = calc_extent(items);
    items.sort_by(|a, b| {
        let ha = hilbert_bbox(a, HILBERT_MAX, &extent);
        let hb = hilbert_bbox(b, HILBERT_MAX, &extent);
        hb.partial_cmp(&ha).unwrap() // ha > hb
    });
}

#[allow(non_snake_case)]
/// Packed Hilbert R-Tree
pub struct PackedRTree {
    _extent: NodeItem,
    _nodeItems: Vec<NodeItem>,
    _numItems: u64, // TODO: usize ?
    _numNodes: u64, // TODO: usize ?
    _nodeSize: u16,
    _levelBounds: Vec<(u64, u64)>, // TODO: (usize, usize) ?
}

#[allow(non_snake_case)]
impl PackedRTree {
    pub const DEFAULT_NODE_SIZE: u16 = 16;

    fn init(&mut self, nodeSize: u16) {
        assert!(nodeSize >= 2, "Node size must be at least 2");
        assert!(self._numItems > 0, "Cannot create empty tree");
        self._nodeSize = cmp::min(cmp::max(nodeSize, 2u16), 65535u16);
        self._levelBounds = PackedRTree::generateLevelBounds(self._numItems, self._nodeSize);
        self._numNodes = self._levelBounds.first().unwrap().1;
        self._nodeItems = vec![NodeItem::create(0); self._numNodes as usize];
    }

    fn generateLevelBounds(numItems: u64, nodeSize: u16) -> Vec<(u64, u64)> {
        assert!(nodeSize >= 2, "Node size must be at least 2");
        assert!(numItems > 0, "Cannot create empty tree");
        assert!(
            numItems <= u64::MAX - ((numItems / nodeSize as u64) * 2),
            "Number of items too large"
        );

        // number of nodes per level in bottom-up order
        let mut levelNumNodes: Vec<u64> = Vec::new();
        let mut n = numItems;
        let mut numNodes = n;
        levelNumNodes.push(n);
        loop {
            n = (n + nodeSize as u64 - 1) / nodeSize as u64;
            numNodes += n;
            levelNumNodes.push(n);
            if n == 1 {
                break;
            }
        }
        // bounds per level in reversed storage order (top-down)
        let mut levelOffsets: Vec<u64> = Vec::new();
        n = numNodes;
        for size in &levelNumNodes {
            levelOffsets.push(n - size);
            n -= size;
        }
        levelOffsets.reverse();
        levelNumNodes.reverse();
        let mut levelBounds = Vec::new();
        for i in 0..levelNumNodes.len() {
            levelBounds.push((levelOffsets[i], levelOffsets[i] + levelNumNodes[i]));
        }
        levelBounds.reverse();
        levelBounds
    }

    fn generateNodes(&mut self) {
        for i in 0..self._levelBounds.len() - 1 {
            let mut pos = self._levelBounds[i].0 as usize;
            let end = self._levelBounds[i].1 as usize;
            let mut newpos = self._levelBounds[i + 1].0 as usize;
            while pos < end {
                let mut node = NodeItem::create(pos as u64);
                for _j in 0..self._nodeSize {
                    if pos >= end {
                        break;
                    }
                    node.expand(&self._nodeItems[pos]);
                    pos += 1;
                }
                self._nodeItems[newpos] = node;
                newpos += 1;
            }
        }
    }

    fn read_data(&mut self, data: &mut dyn Read) {
        let mut n = NodeItem::create(0);
        let buf: &mut [u8] = unsafe {
            std::slice::from_raw_parts_mut(&mut n as *mut _ as *mut u8, size_of::<NodeItem>())
        };
        for i in 0..self._numNodes as usize {
            data.read_exact(buf).unwrap();
            self._nodeItems[i] = n.clone();
            self._extent.expand(&n);
        }
    }

    pub fn search(&self, minX: f64, minY: f64, maxX: f64, maxY: f64) -> Vec<SearchResultItem> {
        let leafNodesOffset = self._levelBounds.first().unwrap().0 as usize;
        let n = NodeItem::new(minX, minY, maxX, maxY);
        let mut results = Vec::new();
        let mut queue = HashMap::new(); // C++: std::unordered_map
        queue.insert(0, self._levelBounds.len() - 1);
        while queue.len() != 0 {
            let next = queue.iter().next().unwrap();
            let nodeIndex = *next.0;
            let level = *next.1;
            queue.remove(&nodeIndex);
            let isLeafNode = nodeIndex >= self._numNodes as usize - self._numItems as usize;
            // find the end index of the node
            let end = cmp::min(
                nodeIndex + self._nodeSize as usize,
                self._levelBounds[level].1 as usize,
            );
            // search through child nodes
            for pos in nodeIndex..end {
                let nodeItem = &self._nodeItems[pos];
                if !n.intersects(&nodeItem) {
                    continue;
                }
                if isLeafNode {
                    results.push(SearchResultItem {
                        offset: nodeItem.offset as usize,
                        index: pos - leafNodesOffset,
                    });
                } else {
                    queue.insert(nodeItem.offset as usize, level - 1);
                }
            }
        }
        results
    }

    pub fn size(num_items: u64, node_size: u16) -> usize {
        assert!(node_size >= 2, "Node size must be at least 2");
        assert!(num_items > 0, "Cannot create empty tree");
        let node_size_min = cmp::min(cmp::max(node_size, 2), 65535) as u64;
        // limit so that resulting size in bytes can be represented by uint64_t
        assert!(
            num_items <= 1 << 56,
            "Number of items must be less than 2^56"
        );
        let mut n = num_items;
        let mut num_nodes = n;
        loop {
            n = (n + node_size_min - 1) / node_size_min;
            num_nodes += n;
            if n == 1 {
                break;
            }
        }
        num_nodes as usize * size_of::<NodeItem>()
    }

    pub fn build(nodes: &Vec<NodeItem>, extent: &NodeItem, nodeSize: u16) -> PackedRTree {
        let mut tree = PackedRTree {
            _extent: extent.clone(),
            _nodeItems: Vec::new(),
            _numItems: nodes.len() as u64,
            _numNodes: 0,
            _nodeSize: 0,
            _levelBounds: Vec::new(),
        };
        tree.init(nodeSize);
        for i in 0..tree._numItems {
            tree._nodeItems[(tree._numNodes - tree._numItems + i) as usize] =
                nodes[i as usize].clone();
        }
        tree.generateNodes();
        tree
    }

    pub fn from_buf(data: &mut dyn Read, num_items: u64, nodeSize: u16) -> PackedRTree {
        let mut tree = PackedRTree {
            _extent: NodeItem::create(0),
            _nodeItems: Vec::new(),
            _numItems: num_items,
            _numNodes: 0,
            _nodeSize: 0,
            _levelBounds: Vec::new(),
        };
        tree.init(nodeSize);
        tree.read_data(data);
        tree
    }

    pub fn stream_write(&self, out: &mut dyn Write) -> std::io::Result<()> {
        let buf: &[u8] = unsafe {
            std::slice::from_raw_parts(
                self._nodeItems.as_ptr() as *const u8,
                self._nodeItems.len() * size_of::<NodeItem>(),
            )
        };
        out.write_all(buf)
    }
}

#[test]
fn tree_2items() {
    let mut nodes = Vec::new();
    nodes.push(NodeItem::new(0.0, 0.0, 1.0, 1.0));
    nodes.push(NodeItem::new(2.0, 2.0, 3.0, 3.0));
    let extent = calc_extent(&nodes);
    assert_eq!(extent, NodeItem::new(0.0, 0.0, 3.0, 3.0));
    assert!(nodes[0].intersects(&NodeItem::new(0.0, 0.0, 1.0, 1.0)));
    assert!(nodes[1].intersects(&NodeItem::new(2.0, 2.0, 3.0, 3.0)));
    hilbert_sort(&mut nodes);
    let mut offset = 0;
    for mut node in &mut nodes {
        node.offset = offset as u64;
        offset += size_of::<NodeItem>();
    }
    assert!(nodes[1].intersects(&NodeItem::new(0.0, 0.0, 1.0, 1.0)));
    assert!(nodes[0].intersects(&NodeItem::new(2.0, 2.0, 3.0, 3.0)));
    let tree = PackedRTree::build(&nodes, &extent, PackedRTree::DEFAULT_NODE_SIZE);
    let list = tree.search(0.0, 0.0, 1.0, 1.0);
    assert_eq!(list.len(), 1);
    assert!(nodes[list[0].index as usize].intersects(&NodeItem::new(0.0, 0.0, 1.0, 1.0)));
}

#[test]
fn tree_19items_roundtrip_stream_search() {
    let mut nodes = Vec::new();
    nodes.push(NodeItem::new(0.0, 0.0, 1.0, 1.0));
    nodes.push(NodeItem::new(2.0, 2.0, 3.0, 3.0));
    nodes.push(NodeItem::new(100.0, 100.0, 110.0, 110.0));
    nodes.push(NodeItem::new(101.0, 101.0, 111.0, 111.0));
    nodes.push(NodeItem::new(102.0, 102.0, 112.0, 112.0));
    nodes.push(NodeItem::new(103.0, 103.0, 113.0, 113.0));
    nodes.push(NodeItem::new(104.0, 104.0, 114.0, 114.0));
    nodes.push(NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0));
    nodes.push(NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0));
    nodes.push(NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0));
    nodes.push(NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0));
    nodes.push(NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0));
    nodes.push(NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0));
    nodes.push(NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0));
    nodes.push(NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0));
    nodes.push(NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0));
    nodes.push(NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0));
    nodes.push(NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0));
    nodes.push(NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0));
    let extent = calc_extent(&nodes);
    hilbert_sort(&mut nodes);
    let mut offset = 0;
    for mut node in &mut nodes {
        node.offset = offset as u64;
        offset += size_of::<NodeItem>();
    }
    let tree = PackedRTree::build(&nodes, &extent, PackedRTree::DEFAULT_NODE_SIZE);
    let list = tree.search(102.0, 102.0, 103.0, 103.0);
    assert_eq!(list.len(), 4);
    for i in 0..list.len() {
        assert!(
            nodes[list[i].index as usize].intersects(&NodeItem::new(102.0, 102.0, 103.0, 103.0))
        );
    }
    let mut tree_data: Vec<u8> = Vec::new();
    let res = tree.stream_write(&mut tree_data);
    assert!(res.is_ok());
    assert_eq!(tree_data.len(), (nodes.len() + 3) * 40);
    assert_eq!(size_of::<NodeItem>(), 40);

    let tree2 = PackedRTree::from_buf(
        &mut &tree_data[..],
        nodes.len() as u64,
        PackedRTree::DEFAULT_NODE_SIZE,
    );
    let list = tree2.search(102.0, 102.0, 103.0, 103.0);
    assert_eq!(list.len(), 4);
    for i in 0..list.len() {
        assert!(
            nodes[list[i].index as usize].intersects(&NodeItem::new(102.0, 102.0, 103.0, 103.0))
        );
    }

    // auto readNode = [data] (uint8_t *buf, uint32_t i, uint32_t s) {
    //     //std::cout << "i: " << i << std::endl;
    //     std::copy(data + i, data + i + s, buf);
    // };
    // auto list3 = PackedRTree::streamSearch(nodes.size(), 16, {102, 102, 103, 103}, readNode);
    // REQUIRE(list3.size() == 4);
    // for (uint32_t i = 0; i < list3.size(); i++) {
    //     REQUIRE(nodes[list3[i].index].intersects({102, 102, 103, 103}) == true);
    // }
}

#[test]
fn tree_100_000_items_in_denmark() {
    use rand::distributions::{Distribution, Uniform};

    let unifx = Uniform::from(466379..708929);
    let unify = Uniform::from(6096801..6322352);
    let mut rng = rand::thread_rng();

    let mut nodes = Vec::new();
    for _ in 0..100000 {
        let x = unifx.sample(&mut rng) as f64;
        let y = unify.sample(&mut rng) as f64;
        nodes.push(NodeItem::new(x, y, x, y));
    }

    let extent = calc_extent(&nodes);
    hilbert_sort(&mut nodes);
    let tree = PackedRTree::build(&nodes, &extent, PackedRTree::DEFAULT_NODE_SIZE);
    let list = tree.search(690407.0, 6063692.0, 811682.0, 6176467.0);

    for i in 0..list.len() {
        assert!(nodes[list[i].index]
            .intersects(&NodeItem::new(690407.0, 6063692.0, 811682.0, 6176467.0)));
    }

    let mut tree_data: Vec<u8> = Vec::new();
    let res = tree.stream_write(&mut tree_data);
    assert!(res.is_ok());

    // auto list2 = PackedRTree::streamSearch(nodes.size(), 16, {690407, 6063692, 811682, 6176467}, readNode);
    // for (uint64_t i = 0; i < list2.size(); i++)
    //     CHECK(nodes[list2[i].index].intersects({690407, 6063692, 811682, 6176467}) == true);
}
