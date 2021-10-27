use flatgeobuf::*;
use std::io::{BufWriter, Write};
use tempfile::tempfile;

#[test]
fn write_file() -> std::io::Result<()> {
    let mut file = BufWriter::new(tempfile()?);
    let points = [[1.0, 1.0], [2.0, 2.0]];

    const MAGIC_BYTES: [u8; 8] = [b'f', b'g', b'b', 3, b'f', b'g', b'b', 0];
    file.write(&MAGIC_BYTES)?;

    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let column_args = ColumnArgs {
        name: Some(fbb.create_string("STATE_FIPS")),
        type_: ColumnType::String,
        ..Default::default()
    };
    let column = Column::create(&mut fbb, &column_args);
    let header_args = HeaderArgs {
        name: Some(fbb.create_string("Test1")),
        geometry_type: GeometryType::Point,
        columns: Some(fbb.create_vector(&[column])),
        features_count: points.len() as u64,
        index_node_size: 0,
        ..Default::default()
    };

    let header = Header::create(&mut fbb, &header_args);
    fbb.finish_size_prefixed(header, None);
    let buf = fbb.finished_data();
    file.write(&buf)?;

    for point in points {
        let mut fbb = flatbuffers::FlatBufferBuilder::new();
        let xy = fbb.create_vector(&point);
        let g = Geometry::create(
            &mut fbb,
            &GeometryArgs {
                xy: Some(xy),
                ..Default::default()
            },
        );
        let f = Feature::create(
            &mut fbb,
            &FeatureArgs {
                geometry: Some(g),
                ..Default::default()
            },
        );
        fbb.finish_size_prefixed(f, None);
        let buf = fbb.finished_data();
        assert_eq!(buf.len(), 64);

        file.write(&buf)?;
    }

    Ok(())
}

#[test]
fn verify_header() {
    let mut builder = flatbuffers::FlatBufferBuilder::with_capacity(1024);
    builder.start_vector::<u8>(0);
    let empty_vec = builder.end_vector(0);
    let header_args = HeaderArgs {
        name: Some(builder.create_string("triangle")),
        envelope: Some(builder.create_vector(&[0.0, 0.0, 9.0, 9.0])),
        geometry_type: GeometryType::Triangle,
        columns: Some(empty_vec),
        features_count: 1,
        ..Default::default()
    };
    let header = Header::create(&mut builder, &header_args);
    builder.finish_size_prefixed(header, None);
    let buf = builder.finished_data();

    // verify
    let header = size_prefixed_root_as_header(&buf).unwrap();
    assert_eq!(header.features_count(), 1);

    assert!(
        root_as_header(&buf[4..]).is_err(),
        "Verfication without size prefix fails"
    );
}
