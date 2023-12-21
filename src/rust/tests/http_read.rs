#[cfg(feature = "http")]
mod http {

    use flatgeobuf::*;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn http_read() -> Result<()> {
        let url = "https://github.com/flatgeobuf/flatgeobuf/raw/master/test/data/countries.fgb";
        let fgb = HttpFgbReader::open(url).await?;
        assert_eq!(fgb.header().geometry_type(), GeometryType::MultiPolygon);
        assert_eq!(fgb.header().features_count(), 179);
        let mut fgb = fgb.select_all().await?;
        let feature = fgb.next().await?.unwrap();
        let props = feature.properties()?;
        assert_eq!(props["name"], "Antarctica".to_string());
        Ok(())
    }

    #[tokio::test]
    async fn http_bbox_read() -> Result<()> {
        let url = "https://github.com/flatgeobuf/flatgeobuf/raw/master/test/data/countries.fgb";
        let fgb = HttpFgbReader::open(url).await?;
        assert_eq!(fgb.header().geometry_type(), GeometryType::MultiPolygon);
        assert_eq!(fgb.header().features_count(), 179);
        let mut fgb = fgb.select_bbox(8.8, 47.2, 9.5, 55.3).await?;
        let feature = fgb.next().await?.unwrap();
        let props = feature.properties()?;
        assert_eq!(props["name"], "Denmark".to_string());
        Ok(())
    }

    #[tokio::test]
    async fn http_bbox_read_last_feature() -> Result<()> {
        let url = "https://github.com/flatgeobuf/flatgeobuf/raw/master/test/data/countries.fgb";
        let fgb = HttpFgbReader::open(url).await.unwrap();
        assert_eq!(fgb.header().features_count(), 179);
        let mut feature_iter = fgb.select_bbox(-180.0, -90.0, 180.0, 90.0).await?;
        let mut count = 0;
        while let Some(_next) = feature_iter.next().await? {
            count += 1;
        }
        assert_eq!(count, 179);
        Ok(())
    }

    #[tokio::test]
    async fn http_read_unknown_feature_count() -> Result<()> {
        let url = "https://github.com/flatgeobuf/flatgeobuf/raw/master/test/data/unknown_feature_count.fgb";
        let fgb = HttpFgbReader::open(url).await?;
        assert_eq!(fgb.header().features_count(), 0);
        let mut fgb = fgb.select_all().await?;
        assert_eq!(fgb.features_count(), None);
        let feature = fgb.next().await?;
        assert!(feature.is_none()); // TODO: support reading unknown feature count

        let url = "https://github.com/flatgeobuf/flatgeobuf/raw/master/test/data/unknown_feature_count.fgb";
        let fgb = HttpFgbReader::open(url)
            .await?
            .select_bbox(8.8, 47.2, 9.5, 55.3)
            .await;
        assert_eq!(fgb.err().unwrap().to_string(), "Index missing");
        Ok(())
    }

    #[tokio::test]
    async fn http_bbox_big() -> Result<()> {
        let url = "https://pkg.sourcepole.ch/osm-buildings-ch.fgb";
        let fgb = HttpFgbReader::open(url).await?;
        assert_eq!(fgb.header().geometry_type(), GeometryType::MultiPolygon);
        assert_eq!(fgb.header().features_count(), 2396905);
        let mut fgb = fgb
            .select_bbox(8.522086, 47.363333, 8.553521, 47.376020)
            .await?;
        let feature = fgb.next().await?.unwrap();
        let props = feature.properties()?;
        assert_eq!(props["building"], "residential".to_string());
        Ok(())
    }

    #[tokio::test]
    async fn http_err() {
        let url = "https://github.com/flatgeobuf/flatgeobuf/raw/master/test/data/wrong.fgb";
        let fgb = HttpFgbReader::open(url).await;
        assert_eq!(
            fgb.err().unwrap().to_string(),
            "http status 404".to_string()
        );

        let url = "https://wrong.example.com/countries.fgb";
        let fgb = HttpFgbReader::open(url).await;
        let error_text = fgb.err().unwrap().to_string();
        let expected_error_text = "error trying to connect";
        assert!(
            error_text.contains(expected_error_text),
            "expected to find {expected_error_text} in {error_text}"
        );
    }
}
