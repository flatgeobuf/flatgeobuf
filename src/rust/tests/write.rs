use flatgeobuf::*;

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
    finish_size_prefixed_header_buffer(&mut builder, header);
    let buf = builder.finished_data();

    // verify
    let header = size_prefixed_root_as_header(&buf).unwrap();
    assert_eq!(header.features_count(), 1);

    assert!(
        root_as_header(&buf[4..]).is_err(),
        "Verfication without size prefix fails"
    );
}

#[test]
fn write_column() {
    let mut builder = flatbuffers::FlatBufferBuilder::with_capacity(1024);
    let column_args = ColumnArgs {
        name: Some(builder.create_string("STATE_FIPS")),
        type_: ColumnType::String,
        ..Default::default()
    };
    let column = Column::create(&mut builder, &column_args);
    let header_args = HeaderArgs {
        name: Some(builder.create_string("Test1")),
        geometry_type: GeometryType::MultiPolygon,
        columns: Some(builder.create_vector(&[column])),
        features_count: 1,
        index_node_size: 0,
        ..Default::default()
    };
    let header = Header::create(&mut builder, &header_args);
    finish_header_buffer(&mut builder, header);
    let buf = builder.finished_data();
    let header = root_as_header(&buf).unwrap();
    assert_eq!(header.features_count(), 1);
}
