use flatgeobuf::*;
use std::env::temp_dir;
use std::fs::File;
use std::io::{BufReader, Read, Write};

#[test]
fn verify_header() {
    let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
    builder.start_vector::<u8>(0);
    let empty_vec = builder.end_vector(0);
    // Header { name: Some("triangle"), envelope: Some([0.0, 0.0, 9.0, 9.0]), geometry_type: Triangle, hasZ: false, hasM: false, hasT: false, hasTM: false, columns: Some([]), features_count: 1, index_node_size: 16, crs: None, title: None, description: None, metadata: None }
    let header_args = HeaderArgs {
        name: Some(builder.create_string("triangle")),
        envelope: Some(builder.create_vector(&[0.0, 0.0, 9.0, 9.0])),
        geometry_type: GeometryType::Triangle,
        columns: Some(empty_vec),
        features_count: 1,
        ..Default::default()
    };
    let header = Header::create(&mut builder, &header_args);
    finish_header_buffer(&mut builder, header);
    let buf = builder.finished_data();
    let header = root_as_header(&buf).unwrap();
    assert_eq!(header.features_count(), 1);

    println!("{:?}", &buf);
    write_to_file(buf, &tmp_fname("header.bin"));
    write_fgb_to_file(buf, &tmp_fname("header.fgb"));

    // read into another buffer and decode again
    let mut header_buf = Vec::with_capacity(buf.len());
    let mut reader = BufReader::new(buf);
    reader.read_to_end(&mut header_buf).unwrap();
    let header = root_as_header(&header_buf).unwrap();
    assert_eq!(header.features_count(), 1);

    // Shift buffer to check alignement
    header_buf.insert(0, 0);
    let header = root_as_header(&header_buf[1..]).unwrap();
    assert_eq!(header.features_count(), 1);

    let buf = read_from_file(&tmp_fname("header.bin"));
    let header = root_as_header(&buf).unwrap();
    assert_eq!(header.features_count(), 1);

    let buf = reader_header(&tmp_fname("header.fgb"));
    println!("{:?}", &buf);
    // [28, 0, 0, 0, 0, 0, 22, 0, 32, 0, 8, 0, 12, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 20, 0, 22, 0, 0, 0, 0, 0, 0, 17, 64, 0, 0, 0, 20, 0, 0, 0, 72, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34, 64, 0, 0, 0, 0, 0, 0, 34, 64, 0, 0, 0, 0, 8, 0, 0, 0, 116, 114, 105, 97, 110, 103, 108, 101, 0, 0, 0, 0, 0, 0, 0, 0]
    let header = root_as_header(&buf).unwrap();
    assert_eq!(header.features_count(), 1);

    let buf = read_from_file(&tmp_fname("header.fgb"));
    let header = root_as_header(&buf[12..]).unwrap();
    assert_eq!(header.features_count(), 1);

    assert!(size_prefixed_root_as_header(&buf[8..]).is_err(), "Verfication with size prefix fails");
}

fn tmp_fname(fname: &str) -> String {
    let mut tmpfile = temp_dir();
    tmpfile.push(fname);
    tmpfile.to_str().unwrap().to_string()
}

fn write_to_file(data: &[u8], fname: &str) {
    let mut file = File::create(fname).expect("create failed");
    file.write_all(data).expect("write failed");
}

fn write_fgb_to_file(data: &[u8], fname: &str) {
    let mut file = File::create(fname).expect("create failed");
    file.write_all(&MAGIC_BYTES).expect("write failed");
    file.write_all(&(data.len() as u32).to_le_bytes())
        .expect("write failed");
    file.write_all(data).expect("write failed");
}

fn read_from_file(fname: &str) -> Vec<u8> {
    let mut f = File::open(fname).expect("open failed");
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).expect("read failed");
    buf
}

fn reader_header(fname: &str) -> Vec<u8> {
    let f = File::open(fname).expect("open failed");
    let mut reader = BufReader::new(f);

    let mut magic_buf: [u8; 8] = [0; 8];
    reader.read_exact(&mut magic_buf).expect("read failed");
    // assert_eq!(magic_buf, MAGIC_BYTES);

    let mut size_buf: [u8; 4] = [0; 4];
    reader.read_exact(&mut size_buf).expect("read failed");
    let header_size = u32::from_le_bytes(size_buf);

    let mut header_buf = vec![0; header_size as usize];
    reader.read_exact(&mut header_buf).expect("read failed");

    header_buf
}

#[test]
fn write_column() {
    let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(1024);
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

#[test]
fn header_fgb_only() {
    // let f = File::open("../../test/data/surface/triangle.fgb").expect("open failed");
    // let mut reader = BufReader::new(f);

    // let mut magic_buf: [u8; 8] = [0; 8];
    // reader.read_exact(&mut magic_buf).expect("read failed");

    // let mut size_buf: [u8; 4] = [0; 4];
    // reader.read_exact(&mut size_buf).expect("read failed");
    // let header_size = u32::from_le_bytes(size_buf);

    // let mut data = vec![0; header_size as usize];
    // reader.read_exact(&mut data).expect("read failed");

    // let mut file = File::create(tmp_fname("triangle-header.fgb")).expect("create failed");
    // file.write_all(&(data.len() as u32).to_le_bytes())
    //     .expect("write failed");
    // file.write_all(&data).expect("write failed");

    let mut f = File::open("../../test/data/triangle-header.fgb").unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    println!("{:?}", &buf);
    // [116, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 22, 0, 28, 0, 8, 0, 12, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 20, 0, 22, 0, 0, 0, 0, 0, 0, 17, 60, 0, 0, 0, 20, 0, 0, 0, 12, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34, 64, 0, 0, 0, 0, 0, 0, 34, 64, 8, 0, 0, 0, 116, 114, 105, 97, 110, 103, 108, 101, 0, 0, 0, 0]

    let header = size_prefixed_root_as_header(&buf).unwrap();
    println!("{:?}", &header);
    assert_eq!(header.features_count(), 1);

    assert!(root_as_header(&buf[4..]).is_err(), "Verfication without size prefix fails");
}
