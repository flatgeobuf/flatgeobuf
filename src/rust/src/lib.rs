#[allow(dead_code, unused_imports, non_snake_case)]
mod feature_generated;
#[allow(dead_code, unused_imports, non_snake_case)]
mod header_generated;

pub use feature_generated::flat_geobuf::*;
pub use header_generated::flat_geobuf::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Verify size of feature/geometry with point
    fn point() {
        // int size;

        // FlatBufferBuilder fbb1;
        // FlatBufferBuilder fbb2;
        // FlatBufferBuilder fbb3;

        let mut fbb1 = flatbuffers::FlatBufferBuilder::new();

        // std::vector<double> xy;
        // xy.push_back(0);
        // xy.push_back(0);

        // auto g1 = CreateGeometryDirect(fbb1, nullptr, nullptr);
        // fbb1.Finish(g1);
        // size = fbb1.GetSize();
        // REQUIRE(size == 12);

        let g1 = Geometry::create(&mut fbb1, &GeometryArgs::default());
        fbb1.finish(g1, None);
        let buf = fbb1.finished_data();
        let size = buf.len();
        assert_eq!(size, 12);

        // auto g2 = CreateGeometryDirect(fbb2, nullptr, &xy);
        // fbb2.Finish(g2);
        // size = fbb2.GetSize();
        // REQUIRE(size == 40);

        // auto g3 = CreateGeometryDirect(fbb3, nullptr, &xy);
        // auto f = CreateFeatureDirect(fbb3, g3);
        // fbb3.Finish(f);
        // size = fbb3.GetSize();
        // REQUIRE(size == 56);
    }
}
