use flatgeobuf::*;
use std::io::{BufReader, Read, Seek, SeekFrom};

#[test]
fn read_file() -> std::result::Result<(), std::io::Error> {
    let f = std::fs::File::open("../../test/data/countries.fgb")?;
    let mut reader = BufReader::new(f);

    let mut magic_buf: [u8; 8] = [0; 8];
    reader.read_exact(&mut magic_buf)?;
    assert_eq!(magic_buf, MAGIC_BYTES);

    let mut size_buf: [u8; 4] = [0; 4];
    reader.read_exact(&mut size_buf)?;
    let header_size = u32::from_le_bytes(size_buf);
    assert_eq!(header_size, 604);

    let mut header_buf = vec![0; header_size as usize];
    reader.read_exact(&mut header_buf)?;

    let header = get_root_as_header(&header_buf[..]);
    assert_eq!(header.name(), Some("countries"));
    assert!(header.envelope().is_some());
    assert_eq!(header.geometry_type(), GeometryType::MultiPolygon);
    assert_eq!(header.hasZ(), false);
    assert_eq!(header.hasM(), false);
    assert_eq!(header.hasT(), false);
    assert_eq!(header.hasTM(), false);
    // assert_eq!(header.columns(), ...);
    assert_eq!(header.features_count(), 179);
    assert_eq!(header.index_node_size(), 16);
    assert!(header.crs().is_some());

    // Skip index
    let index_size = packed_rtree_size(header.features_count(), header.index_node_size());
    reader.seek(SeekFrom::Current(index_size as i64))?;

    // Read first feature
    reader.read_exact(&mut size_buf)?;
    let feature_size = u32::from_le_bytes(size_buf);
    assert_eq!(feature_size, 10804);
    let mut feature_buf = vec![0; feature_size as usize];
    reader.read_exact(&mut feature_buf)?;

    let feature = get_root_as_feature(&feature_buf[..]);
    assert!(feature.geometry().is_some());
    let geometry = feature.geometry().unwrap();
    assert_eq!(geometry.type_(), GeometryType::MultiPolygon);
    assert!(feature.properties().is_some());
    // assert_eq!(feature.columns(), ...);

    Ok(())
}

fn packed_rtree_size(num_items: u64, node_size: u16) -> u64 {
    let node_size_min = node_size as u64;
    let mut n = num_items;
    let mut num_nodes = n;
    loop {
        n = (n + node_size_min - 1) / node_size_min;
        num_nodes += n;
        if n == 1 {
            break;
        }
    }
    num_nodes * 40
}
// uint64_t PackedRTree::size(const uint64_t numItems, const uint16_t nodeSize)
// {
//     if (nodeSize < 2)
//         throw std::invalid_argument("Node size must be at least 2");
//     if (numItems == 0)
//         throw std::invalid_argument("Number of items must be greater than 0");
//     const uint16_t nodeSizeMin = std::min(std::max(nodeSize, static_cast<uint16_t>(2)), static_cast<uint16_t>(65535));
//     // limit so that resulting size in bytes can be represented by uint64_t
//     if (numItems > static_cast<uint64_t>(1) << 56)
//         throw std::overflow_error("Number of items must be less than 2^56");
//     uint64_t n = numItems;
//     uint64_t numNodes = n;
//     do {
//         n = (n + nodeSizeMin - 1) / nodeSizeMin;
//         numNodes += n;
//     } while (n != 1);
//     return numNodes * sizeof(NodeItem);
// }
