//! Create and read a [packed Hilbert R-Tree](https://en.wikipedia.org/wiki/Hilbert_R-tree#Packed_Hilbert_R-trees)
//! to enable fast bounding box spatial filtering.

#[cfg(feature = "http")]
use crate::http_reader::from_http_err;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use geozero::error::{GeozeroError, Result};
#[cfg(feature = "http")]
use http_range_client::BufferedHttpRangeClient;
use std::cmp::{max, min};
use std::collections::VecDeque;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::mem::size_of;

#[derive(Clone, PartialEq, Debug)]
#[repr(C)]
/// R-Tree node
pub struct NodeItem {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
    /// Byte offset in feature data section
    pub offset: u64,
}

impl NodeItem {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> NodeItem {
        NodeItem {
            min_x,
            min_y,
            max_x,
            max_y,
            offset: 0,
        }
    }

    pub fn create(offset: u64) -> NodeItem {
        NodeItem {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
            offset,
        }
    }

    pub fn from_reader(mut rdr: impl Read) -> Result<Self> {
        Ok(NodeItem {
            min_x: rdr.read_f64::<LittleEndian>()?,
            min_y: rdr.read_f64::<LittleEndian>()?,
            max_x: rdr.read_f64::<LittleEndian>()?,
            max_y: rdr.read_f64::<LittleEndian>()?,
            offset: rdr.read_u64::<LittleEndian>()?,
        })
    }

    fn from_bytes(raw: &[u8]) -> Result<Self> {
        Self::from_reader(&mut Cursor::new(raw))
    }

    pub fn write<W: Write>(&self, wtr: &mut W) -> std::io::Result<()> {
        wtr.write_f64::<LittleEndian>(self.min_x)?;
        wtr.write_f64::<LittleEndian>(self.min_y)?;
        wtr.write_f64::<LittleEndian>(self.max_x)?;
        wtr.write_f64::<LittleEndian>(self.max_y)?;
        wtr.write_u64::<LittleEndian>(self.offset)?;
        Ok(())
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn sum(mut a: NodeItem, b: &NodeItem) -> NodeItem {
        a.expand(b);
        a
    }

    pub fn expand(&mut self, r: &NodeItem) {
        if r.min_x < self.min_x {
            self.min_x = r.min_x;
        }
        if r.min_y < self.min_y {
            self.min_y = r.min_y;
        }
        if r.max_x > self.max_x {
            self.max_x = r.max_x;
        }
        if r.max_y > self.max_y {
            self.max_y = r.max_y;
        }
    }

    pub fn expand_xy(&mut self, x: f64, y: f64) {
        if x < self.min_x {
            self.min_x = x;
        }
        if y < self.min_y {
            self.min_y = y;
        }
        if x > self.max_x {
            self.max_x = x;
        }
        if y > self.max_y {
            self.max_y = y;
        }
    }

    pub fn intersects(&self, r: &NodeItem) -> bool {
        if self.max_x < r.min_x {
            return false;
        }
        if self.max_y < r.min_y {
            return false;
        }
        if self.min_x > r.max_x {
            return false;
        }
        if self.min_y > r.max_y {
            return false;
        }
        true
    }
}

/// Read full capacity of vec from data stream
fn read_node_vec(node_items: &mut Vec<NodeItem>, mut data: impl Read) -> Result<()> {
    node_items.clear();
    for _ in 0..node_items.capacity() {
        node_items.push(NodeItem::from_reader(&mut data)?);
    }
    Ok(())
}

/// Read partial item vec from data stream
fn read_node_items<R: Read + Seek>(
    data: &mut R,
    base: u64,
    node_index: usize,
    length: usize,
) -> Result<Vec<NodeItem>> {
    let mut node_items = Vec::with_capacity(length);
    data.seek(SeekFrom::Start(
        base + (node_index * size_of::<NodeItem>()) as u64,
    ))?;
    read_node_vec(&mut node_items, data)?;
    Ok(node_items)
}

/// Read partial item vec from http
#[cfg(feature = "http")]
async fn read_http_node_items(
    client: &mut BufferedHttpRangeClient,
    min_req_size: usize,
    base: usize,
    node_index: usize,
    length: usize,
) -> Result<Vec<NodeItem>> {
    let begin = base + node_index * size_of::<NodeItem>();
    let len = length * size_of::<NodeItem>();
    let bytes = client
        .get_range(begin, len, min_req_size)
        .await
        .map_err(from_http_err)?;

    let mut node_items = Vec::with_capacity(length);
    for i in 0..length {
        node_items.push(NodeItem::from_bytes(
            &bytes[i * size_of::<NodeItem>()..(i + 1) * size_of::<NodeItem>()],
        )?);
    }
    Ok(node_items)
}

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
fn hilbert(x: u32, y: u32) -> u32 {
    let mut a = x ^ y;
    let mut b = 0xFFFF ^ a;
    let mut c = 0xFFFF ^ (x | y);
    let mut d = x & (y ^ 0xFFFF);

    let mut aa = a | (b >> 1);
    let mut bb = (a >> 1) ^ a;
    let mut cc = ((c >> 1) ^ (b & (d >> 1))) ^ c;
    let mut dd = ((a & (c >> 1)) ^ (d >> 1)) ^ d;

    a = aa;
    b = bb;
    c = cc;
    d = dd;
    aa = (a & (a >> 2)) ^ (b & (b >> 2));
    bb = (a & (b >> 2)) ^ (b & ((a ^ b) >> 2));
    cc ^= (a & (c >> 2)) ^ (b & (d >> 2));
    dd ^= (b & (c >> 2)) ^ ((a ^ b) & (d >> 2));

    a = aa;
    b = bb;
    c = cc;
    d = dd;
    aa = (a & (a >> 4)) ^ (b & (b >> 4));
    bb = (a & (b >> 4)) ^ (b & ((a ^ b) >> 4));
    cc ^= (a & (c >> 4)) ^ (b & (d >> 4));
    dd ^= (b & (c >> 4)) ^ ((a ^ b) & (d >> 4));

    a = aa;
    b = bb;
    c = cc;
    d = dd;
    cc ^= (a & (c >> 8)) ^ (b & (d >> 8));
    dd ^= (b & (c >> 8)) ^ ((a ^ b) & (d >> 8));

    a = cc ^ (cc >> 1);
    b = dd ^ (dd >> 1);

    let mut i0 = x ^ y;
    let mut i1 = b | (0xFFFF ^ (i0 | a));

    i0 = (i0 | (i0 << 8)) & 0x00FF00FF;
    i0 = (i0 | (i0 << 4)) & 0x0F0F0F0F;
    i0 = (i0 | (i0 << 2)) & 0x33333333;
    i0 = (i0 | (i0 << 1)) & 0x55555555;

    i1 = (i1 | (i1 << 8)) & 0x00FF00FF;
    i1 = (i1 | (i1 << 4)) & 0x0F0F0F0F;
    i1 = (i1 | (i1 << 2)) & 0x33333333;
    i1 = (i1 | (i1 << 1)) & 0x55555555;

    (i1 << 1) | i0
}

fn hilbert_bbox(r: &NodeItem, hilbert_max: u32, extent: &NodeItem) -> u32 {
    // calculate bbox center and scale to hilbert_max
    let x = (hilbert_max as f64 * ((r.min_x + r.max_x) / 2.0 - extent.min_x) / extent.width())
        .floor() as u32;
    let y = (hilbert_max as f64 * ((r.min_y + r.max_y) / 2.0 - extent.min_y) / extent.height())
        .floor() as u32;
    hilbert(x, y)
}

pub fn hilbert_sort(items: &mut [NodeItem], extent: &NodeItem) {
    items.sort_by(|a, b| {
        let ha = hilbert_bbox(a, HILBERT_MAX, extent);
        let hb = hilbert_bbox(b, HILBERT_MAX, extent);
        hb.partial_cmp(&ha).unwrap() // ha > hb
    });
}

pub fn calc_extent(nodes: &[NodeItem]) -> NodeItem {
    nodes.iter().fold(NodeItem::create(0), |mut a, b| {
        a.expand(b);
        a
    })
}

/// Packed Hilbert R-Tree
pub struct PackedRTree {
    extent: NodeItem,
    node_items: Vec<NodeItem>,
    num_items: usize,
    num_nodes: usize,
    node_size: u16,
    level_bounds: Vec<(usize, usize)>,
}

impl PackedRTree {
    pub const DEFAULT_NODE_SIZE: u16 = 16;

    fn init(&mut self, node_size: u16) -> Result<()> {
        assert!(node_size >= 2, "Node size must be at least 2");
        assert!(self.num_items > 0, "Cannot create empty tree");
        self.node_size = min(max(node_size, 2u16), 65535u16);
        self.level_bounds = PackedRTree::generate_level_bounds(self.num_items, self.node_size);
        self.num_nodes = self
            .level_bounds
            .first()
            .ok_or(GeozeroError::GeometryIndex)?
            .1;
        self.node_items = vec![NodeItem::create(0); self.num_nodes]; // Quite slow!
        Ok(())
    }

    fn generate_level_bounds(num_items: usize, node_size: u16) -> Vec<(usize, usize)> {
        assert!(node_size >= 2, "Node size must be at least 2");
        assert!(num_items > 0, "Cannot create empty tree");
        assert!(
            num_items <= usize::MAX - ((num_items / node_size as usize) * 2),
            "Number of items too large"
        );

        // number of nodes per level in bottom-up order
        let mut level_num_nodes: Vec<usize> = Vec::new();
        let mut n = num_items;
        let mut num_nodes = n;
        level_num_nodes.push(n);
        loop {
            n = (n + node_size as usize - 1) / node_size as usize;
            num_nodes += n;
            level_num_nodes.push(n);
            if n == 1 {
                break;
            }
        }
        // bounds per level in reversed storage order (top-down)
        let mut level_offsets: Vec<usize> = Vec::with_capacity(level_num_nodes.len());
        n = num_nodes;
        for size in &level_num_nodes {
            level_offsets.push(n - size);
            n -= size;
        }
        let mut level_bounds = Vec::with_capacity(level_num_nodes.len());
        for i in 0..level_num_nodes.len() {
            level_bounds.push((level_offsets[i], level_offsets[i] + level_num_nodes[i]));
        }
        level_bounds
    }

    fn generate_nodes(&mut self) {
        for i in 0..self.level_bounds.len() - 1 {
            let mut pos = self.level_bounds[i].0;
            let end = self.level_bounds[i].1;
            let mut newpos = self.level_bounds[i + 1].0;
            while pos < end {
                let mut node = NodeItem::create(pos as u64);
                for _j in 0..self.node_size {
                    if pos >= end {
                        break;
                    }
                    node.expand(&self.node_items[pos]);
                    pos += 1;
                }
                self.node_items[newpos] = node;
                newpos += 1;
            }
        }
    }

    fn read_data(&mut self, data: impl Read) -> Result<()> {
        read_node_vec(&mut self.node_items, data)?;
        for node in &self.node_items {
            self.extent.expand(node)
        }
        Ok(())
    }

    #[cfg(feature = "http")]
    async fn read_http(
        &mut self,
        client: &mut BufferedHttpRangeClient,
        index_begin: usize,
    ) -> Result<()> {
        let min_req_size = self.size(); // read full index at once
        let mut pos = index_begin;
        for i in 0..self.num_nodes {
            let bytes = client
                .get_range(pos, size_of::<NodeItem>(), min_req_size)
                .await
                .map_err(from_http_err)?;
            let n = NodeItem::from_bytes(bytes)?;
            self.extent.expand(&n);
            self.node_items[i] = n;
            pos += size_of::<NodeItem>();
        }
        Ok(())
    }

    pub fn build(nodes: &Vec<NodeItem>, extent: &NodeItem, node_size: u16) -> Result<PackedRTree> {
        let mut tree = PackedRTree {
            extent: extent.clone(),
            node_items: Vec::new(),
            num_items: nodes.len(),
            num_nodes: 0,
            node_size: 0,
            level_bounds: Vec::new(),
        };
        tree.init(node_size)?;
        for (i, node) in nodes.iter().take(tree.num_items).cloned().enumerate() {
            tree.node_items[tree.num_nodes - tree.num_items + i] = node;
        }
        tree.generate_nodes();
        Ok(tree)
    }

    pub fn from_buf(data: impl Read, num_items: usize, node_size: u16) -> Result<PackedRTree> {
        let node_size = min(max(node_size, 2u16), 65535u16);
        let level_bounds = PackedRTree::generate_level_bounds(num_items, node_size);
        let num_nodes = level_bounds.first().ok_or(GeozeroError::GeometryIndex)?.1;
        let mut tree = PackedRTree {
            extent: NodeItem::create(0),
            node_items: Vec::with_capacity(num_nodes),
            num_items,
            num_nodes,
            node_size,
            level_bounds,
        };
        tree.read_data(data)?;
        Ok(tree)
    }

    #[cfg(feature = "http")]
    pub async fn from_http(
        client: &mut BufferedHttpRangeClient,
        index_begin: usize,
        num_items: usize,
        node_size: u16,
    ) -> Result<PackedRTree> {
        let mut tree = PackedRTree {
            extent: NodeItem::create(0),
            node_items: Vec::new(),
            num_items,
            num_nodes: 0,
            node_size: 0,
            level_bounds: Vec::new(),
        };
        tree.init(node_size)?;
        tree.read_http(client, index_begin).await?;
        Ok(tree)
    }

    pub fn search(
        &self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> Result<Vec<SearchResultItem>> {
        let leaf_nodes_offset = self
            .level_bounds
            .first()
            .ok_or(GeozeroError::GeometryIndex)?
            .0;
        let n = NodeItem::new(min_x, min_y, max_x, max_y);
        let mut results = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back((0, self.level_bounds.len() - 1));
        while let Some(next) = queue.pop_front() {
            let node_index = next.0;
            let level = next.1;
            let is_leaf_node = node_index >= self.num_nodes - self.num_items;
            // find the end index of the node
            let end = min(
                node_index + self.node_size as usize,
                self.level_bounds[level].1,
            );
            // search through child nodes
            for pos in node_index..end {
                let node_item = &self.node_items[pos];
                if !n.intersects(node_item) {
                    continue;
                }
                if is_leaf_node {
                    results.push(SearchResultItem {
                        offset: node_item.offset as usize,
                        index: pos - leaf_nodes_offset,
                    });
                } else {
                    queue.push_back((node_item.offset as usize, level - 1));
                }
            }
        }
        Ok(results)
    }

    pub fn stream_search<R: Read + Seek>(
        data: &mut R,
        num_items: usize,
        node_size: u16,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> Result<Vec<SearchResultItem>> {
        let item = NodeItem::new(min_x, min_y, max_x, max_y);
        let level_bounds = PackedRTree::generate_level_bounds(num_items, node_size);
        let leaf_nodes_offset = level_bounds.first().ok_or(GeozeroError::GeometryIndex)?.0;
        let num_nodes = level_bounds.first().ok_or(GeozeroError::GeometryIndex)?.1;

        // current position must be start of index
        let index_base = data.stream_position()?;

        // use ordered search queue to make index traversal in sequential order
        let mut queue = VecDeque::new();
        queue.push_back((0, level_bounds.len() - 1));
        let mut results = Vec::new();

        while let Some(next) = queue.pop_front() {
            let node_index = next.0;
            let level = next.1;
            println!("popped next node_index: {node_index}, level: {level}");
            let is_leaf_node = node_index >= num_nodes - num_items;
            // find the end index of the node
            let end = min(node_index + node_size as usize, level_bounds[level].1);
            let length = end - node_index;
            let node_items = read_node_items(data, index_base, node_index, length)?;
            // search through child nodes
            for pos in node_index..end {
                let node_pos = pos - node_index;
                let node_item = &node_items[node_pos];
                if !item.intersects(node_item) {
                    continue;
                }
                if is_leaf_node {
                    let index = pos - leaf_nodes_offset;
                    let offset = node_item.offset as usize;
                    println!("pushing leaf node. index: {index}, offset: {offset}");
                    results.push(SearchResultItem { offset, index });
                } else {
                    let offset = node_item.offset as usize;
                    let prev_level = level - 1;
                    println!("pushing branch node. prev_level: {prev_level}, offset: {offset}");
                    queue.push_back((offset, prev_level));
                }
            }
        }
        // Skip rest of index
        data.seek(SeekFrom::Start(
            index_base + (num_nodes * size_of::<NodeItem>()) as u64,
        ))?;
        Ok(results)
    }

    #[cfg(feature = "http")]
    #[allow(clippy::too_many_arguments)]
    pub async fn http_stream_search(
        client: &mut BufferedHttpRangeClient,
        index_begin: usize,
        num_items: usize,
        node_size: u16,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> Result<Vec<SearchResultItem>> {
        let item = NodeItem::new(min_x, min_y, max_x, max_y);
        let level_bounds = PackedRTree::generate_level_bounds(num_items, node_size);
        let leaf_nodes_offset = level_bounds.first().ok_or(GeozeroError::GeometryIndex)?.0;
        debug!("http_stream_search - index_begin: {index_begin}, num_items: {num_items}, node_size: {node_size}, level_bounds: {level_bounds:?}, GPS bounds:[({min_x}, {min_y}), ({max_x},{max_y})]");

        #[derive(Debug, PartialEq, Eq)]
        struct NodeRange {
            level: usize,
            nodes: std::ops::Range<usize>,
        }

        let mut queue = VecDeque::new();
        queue.push_back(NodeRange {
            nodes: 0..1,
            level: level_bounds.len() - 1,
        });
        let mut results = Vec::new();

        while let Some(next) = queue.pop_front() {
            debug!(
                "popped node: {next:?},  remaining queue len: {}",
                queue.len()
            );
            let node_index = next.nodes.start;
            let is_leaf_node = node_index >= leaf_nodes_offset;
            // find the end index of the nodes
            let end = min(
                next.nodes.end + node_size as usize,
                level_bounds[next.level].1,
            );
            let length = end - node_index;
            let node_items =
                read_http_node_items(client, 0, index_begin, node_index, length).await?;

            // search through child nodes
            for pos in node_index..end {
                let node_pos = pos - node_index;
                let node_item = &node_items[node_pos];
                if !item.intersects(node_item) {
                    continue;
                }
                if is_leaf_node {
                    results.push(SearchResultItem {
                        offset: node_item.offset as usize,
                        index: pos - leaf_nodes_offset,
                    });
                    continue;
                }

                // request up to this many extra bytes if it means we can eliminate an extra request
                let combine_request_threshold = 256 * 1024 / size_of::<NodeItem>();

                // Add node to search recursion
                match (queue.back_mut(), node_item.offset as usize) {
                    // There is an existing node for this level, and it's close to this node.
                    // Merge the ranges to avoid an extra request
                    (Some(tail), offset)
                        if tail.level == next.level - 1
                            && offset < tail.nodes.end + combine_request_threshold =>
                    {
                        debug_assert!(tail.nodes.end < offset);
                        tail.nodes.end = offset;
                    }

                    (tail, offset) => {
                        let node_range = NodeRange {
                            nodes: offset..(offset + 1),
                            level: next.level - 1,
                        };

                        if tail
                            .as_ref()
                            .map(|head| head.level == next.level - 1)
                            .unwrap_or(false)
                        {
                            debug!("requesting new NodeRange for offset: {offset} rather than merging with distant NodeRange: {tail:?}");
                        } else {
                            debug!(
                                "pushing new level for NodeRange: {node_range:?} onto Queue with tail: {:?}",
                                queue.back()
                            );
                        }

                        queue.push_back(node_range);
                    }
                }
            }
        }
        Ok(results)
    }

    pub fn size(&self) -> usize {
        self.num_nodes * size_of::<NodeItem>()
    }

    pub fn index_size(num_items: usize, node_size: u16) -> usize {
        assert!(node_size >= 2, "Node size must be at least 2");
        assert!(num_items > 0, "Cannot create empty tree");
        let node_size_min = min(max(node_size, 2), 65535) as usize;
        // limit so that resulting size in bytes can be represented by uint64_t
        // assert!(
        //     num_items <= 1 << 56,
        //     "Number of items must be less than 2^56"
        // );
        let mut n = num_items;
        let mut num_nodes = n;
        loop {
            n = (n + node_size_min - 1) / node_size_min;
            num_nodes += n;
            if n == 1 {
                break;
            }
        }
        num_nodes * size_of::<NodeItem>()
    }

    /// Write all index nodes
    pub fn stream_write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        for item in &self.node_items {
            item.write(out)?;
        }
        Ok(())
    }

    pub fn extent(&self) -> NodeItem {
        self.extent.clone()
    }
}

mod inspect {
    use super::*;
    use geozero::{ColumnValue, FeatureProcessor};

    impl PackedRTree {
        pub fn process_index<P: FeatureProcessor>(&self, processor: &mut P) -> Result<()> {
            processor.dataset_begin(Some("PackedRTree"))?;
            let mut fid = 0;
            for (levelno, level) in self.level_bounds.iter().rev().enumerate() {
                for pos in level.0..level.1 {
                    let node = &self.node_items[pos];
                    processor.feature_begin(fid)?;
                    processor.properties_begin()?;
                    let _ =
                        processor.property(0, "levelno", &ColumnValue::ULong(levelno as u64))?;
                    let _ = processor.property(1, "pos", &ColumnValue::ULong(pos as u64))?;
                    let _ = processor.property(2, "offset", &ColumnValue::ULong(node.offset))?;
                    processor.properties_end()?;
                    processor.geometry_begin()?;
                    processor.polygon_begin(true, 1, 0)?;
                    processor.linestring_begin(false, 5, 0)?;
                    processor.xy(node.min_x, node.min_y, 0)?;
                    processor.xy(node.min_x, node.max_y, 1)?;
                    processor.xy(node.max_x, node.max_y, 2)?;
                    processor.xy(node.max_x, node.min_y, 3)?;
                    processor.xy(node.min_x, node.min_y, 4)?;
                    processor.linestring_end(false, 0)?;
                    processor.polygon_end(true, 0)?;
                    processor.geometry_end()?;
                    processor.feature_end(fid)?;
                    fid += 1;
                }
            }
            processor.dataset_end()
        }
    }
}

#[test]
fn tree_2items() -> Result<()> {
    let mut nodes = Vec::new();
    nodes.push(NodeItem::new(0.0, 0.0, 1.0, 1.0));
    nodes.push(NodeItem::new(2.0, 2.0, 3.0, 3.0));
    let extent = calc_extent(&nodes);
    assert_eq!(extent, NodeItem::new(0.0, 0.0, 3.0, 3.0));
    assert!(nodes[0].intersects(&NodeItem::new(0.0, 0.0, 1.0, 1.0)));
    assert!(nodes[1].intersects(&NodeItem::new(2.0, 2.0, 3.0, 3.0)));
    hilbert_sort(&mut nodes, &extent);
    let mut offset = 0;
    for node in &mut nodes {
        node.offset = offset;
        offset += size_of::<NodeItem>() as u64;
    }
    assert!(nodes[1].intersects(&NodeItem::new(0.0, 0.0, 1.0, 1.0)));
    assert!(nodes[0].intersects(&NodeItem::new(2.0, 2.0, 3.0, 3.0)));
    let tree = PackedRTree::build(&nodes, &extent, PackedRTree::DEFAULT_NODE_SIZE)?;
    let list = tree.search(0.0, 0.0, 1.0, 1.0)?;
    assert_eq!(list.len(), 1);
    assert!(nodes[list[0].index].intersects(&NodeItem::new(0.0, 0.0, 1.0, 1.0)));
    Ok(())
}

#[test]
fn tree_19items_roundtrip_stream_search() -> Result<()> {
    let mut nodes = vec![
        NodeItem::new(0.0, 0.0, 1.0, 1.0),
        NodeItem::new(2.0, 2.0, 3.0, 3.0),
        NodeItem::new(100.0, 100.0, 110.0, 110.0),
        NodeItem::new(101.0, 101.0, 111.0, 111.0),
        NodeItem::new(102.0, 102.0, 112.0, 112.0),
        NodeItem::new(103.0, 103.0, 113.0, 113.0),
        NodeItem::new(104.0, 104.0, 114.0, 114.0),
        NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0),
        NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0),
        NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0),
        NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0),
        NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0),
        NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0),
        NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0),
        NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0),
        NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0),
        NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0),
        NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0),
        NodeItem::new(10010.0, 10010.0, 10110.0, 10110.0),
    ];

    let extent = calc_extent(&nodes);
    hilbert_sort(&mut nodes, &extent);
    let mut offset = 0;
    for node in &mut nodes {
        node.offset = offset;
        offset += size_of::<NodeItem>() as u64;
    }
    let tree = PackedRTree::build(&nodes, &extent, PackedRTree::DEFAULT_NODE_SIZE)?;
    let list = tree.search(102.0, 102.0, 103.0, 103.0)?;
    assert_eq!(list.len(), 4);

    let indexes: Vec<usize> = list.iter().map(|item| item.index).collect();
    let expected: Vec<usize> = vec![13, 14, 15, 16];
    assert_eq!(indexes, expected);

    let mut tree_data: Vec<u8> = Vec::new();
    let res = tree.stream_write(&mut tree_data);
    assert!(res.is_ok());
    assert_eq!(tree_data.len(), (nodes.len() + 3) * size_of::<NodeItem>());
    assert_eq!(size_of::<NodeItem>(), 40);

    let tree2 = PackedRTree::from_buf(
        &mut &tree_data[..],
        nodes.len(),
        PackedRTree::DEFAULT_NODE_SIZE,
    )?;
    let list = tree2.search(102.0, 102.0, 103.0, 103.0)?;
    assert_eq!(list.len(), 4);

    let indexes: Vec<usize> = list.iter().map(|item| item.index).collect();
    let expected: Vec<usize> = vec![13, 14, 15, 16];
    assert_eq!(indexes, expected);

    let mut reader = Cursor::new(&tree_data);
    let list = PackedRTree::stream_search(
        &mut reader,
        nodes.len(),
        PackedRTree::DEFAULT_NODE_SIZE,
        102.0,
        102.0,
        103.0,
        103.0,
    )?;
    assert_eq!(list.len(), 4);

    let indexes: Vec<usize> = list.iter().map(|item| item.index).collect();
    let expected: Vec<usize> = vec![13, 14, 15, 16];
    assert_eq!(indexes, expected);

    Ok(())
}

#[test]
fn tree_100_000_items_in_denmark() -> Result<()> {
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
    hilbert_sort(&mut nodes, &extent);
    let tree = PackedRTree::build(&nodes, &extent, PackedRTree::DEFAULT_NODE_SIZE)?;
    let list = tree.search(690407.0, 6063692.0, 811682.0, 6176467.0)?;

    for i in 0..list.len() {
        assert!(nodes[list[i].index]
            .intersects(&NodeItem::new(690407.0, 6063692.0, 811682.0, 6176467.0)));
    }

    let mut tree_data: Vec<u8> = Vec::new();
    let res = tree.stream_write(&mut tree_data);
    assert!(res.is_ok());

    let mut reader = Cursor::new(&tree_data);
    let list2 = PackedRTree::stream_search(
        &mut reader,
        nodes.len(),
        PackedRTree::DEFAULT_NODE_SIZE,
        690407.0,
        6063692.0,
        811682.0,
        6176467.0,
    )?;
    assert_eq!(list2.len(), list.len());
    for i in 0..list2.len() {
        assert!(nodes[list2[i].index]
            .intersects(&NodeItem::new(690407.0, 6063692.0, 811682.0, 6176467.0)));
    }
    Ok(())
}

#[test]
fn tree_processing() -> Result<()> {
    use geozero::geojson::GeoJsonWriter;
    use std::io::BufWriter;
    use tempfile::tempfile;

    let mut nodes = Vec::new();
    nodes.push(NodeItem::new(0.0, 0.0, 1.0, 1.0));
    nodes.push(NodeItem::new(2.0, 2.0, 3.0, 3.0));
    let extent = calc_extent(&nodes);
    let mut offset = 0;
    for node in &mut nodes {
        node.offset = offset;
        offset += size_of::<NodeItem>() as u64;
    }
    let tree = PackedRTree::build(&nodes, &extent, PackedRTree::DEFAULT_NODE_SIZE)?;
    let mut fout = BufWriter::new(tempfile()?);
    tree.process_index(&mut GeoJsonWriter::new(&mut fout))?;
    Ok(())
}
