use flatgeobuf::*;

#[test]
/// Verify size of feature/geometry with point
fn point() {
    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let g1 = Geometry::create(&mut fbb, &Default::default());
    fbb.finish(g1, None);
    let buf = fbb.finished_data();
    let size = buf.len();
    assert_eq!(size, 12);

    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let xy = fbb.create_vector(&[0.0, 0.0]);
    let g2 = Geometry::create(
        &mut fbb,
        &GeometryArgs {
            xy: Some(xy),
            ..Default::default()
        },
    );
    fbb.finish(g2, None);
    let buf = fbb.finished_data();
    let size = buf.len();
    assert_eq!(size, 40);

    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let xy = fbb.create_vector(&[0.0, 0.0]);
    let g3 = Geometry::create(
        &mut fbb,
        &GeometryArgs {
            xy: Some(xy),
            ..Default::default()
        },
    );
    let f = Feature::create(
        &mut fbb,
        &FeatureArgs {
            geometry: Some(g3),
            ..Default::default()
        },
    );
    fbb.finish(f, None);
    let buf = fbb.finished_data();
    let size = buf.len();
    assert_eq!(size, 56);
}
