#[cfg(feature = "http")]
mod http {

    use flatgeobuf::*;
    use geozero::error::Result;
    use tokio::runtime::Runtime;

    async fn http_read_async() -> Result<()> {
        let url = "https://github.com/flatgeobuf/flatgeobuf/raw/master/test/data/countries.fgb";
        let mut fgb = HttpFgbReader::open(url).await?;
        assert_eq!(fgb.header().geometry_type(), GeometryType::MultiPolygon);
        assert_eq!(fgb.header().features_count(), 179);
        fgb.select_all().await?;
        let feature = fgb.next().await?.unwrap();
        let props = feature.properties()?;
        assert_eq!(props["name"], "Antarctica".to_string());
        Ok(())
    }

    #[test]
    fn http_read() {
        assert!(Runtime::new().unwrap().block_on(http_read_async()).is_ok());
    }

    async fn http_bbox_read_async() -> Result<()> {
        let url = "https://github.com/flatgeobuf/flatgeobuf/raw/master/test/data/countries.fgb";
        let mut fgb = HttpFgbReader::open(url).await?;
        assert_eq!(fgb.header().geometry_type(), GeometryType::MultiPolygon);
        assert_eq!(fgb.header().features_count(), 179);
        fgb.select_bbox(8.8, 47.2, 9.5, 55.3).await?;
        let feature = fgb.next().await?.unwrap();
        let props = feature.properties()?;
        assert_eq!(props["name"], "Denmark".to_string());
        Ok(())
    }

    #[test]
    fn http_bbox_read() {
        assert!(Runtime::new()
            .unwrap()
            .block_on(http_bbox_read_async())
            .is_ok());
    }

    async fn http_bbox_big_async() -> Result<()> {
        let url = "https://pkg.sourcepole.ch/osm-buildings-ch.fgb";
        let mut fgb = HttpFgbReader::open(url).await?;
        assert_eq!(fgb.header().geometry_type(), GeometryType::MultiPolygon);
        assert_eq!(fgb.header().features_count(), 2396905);
        fgb.select_bbox(8.522086, 47.363333, 8.553521, 47.376020)
            .await?;
        let feature = fgb.next().await?.unwrap();
        let props = feature.properties()?;
        assert_eq!(props["building"], "residential".to_string());
        Ok(())
    }

    #[test]
    #[ignore]
    fn http_bbox_big() {
        assert!(Runtime::new()
            .unwrap()
            .block_on(http_bbox_big_async())
            .is_ok());
    }

    async fn http_err_async() {
        let url = "https://github.com/flatgeobuf/flatgeobuf/raw/master/test/data/wrong.fgb";
        let fgb = HttpFgbReader::open(url).await;
        assert_eq!(
            fgb.err().unwrap().to_string(),
            "http status 404".to_string()
        );
        let url = "http://wrong.sourcepole.ch/countries.fgb";
        let fgb = HttpFgbReader::open(url).await;
        let error_text = fgb.err().unwrap().to_string();
        let expected_error_text = "error trying to connect";
        assert!(
            error_text.contains(expected_error_text),
            "expected to find {} in {}",
            expected_error_text,
            error_text
        );
    }

    #[test]
    fn http_err() {
        Runtime::new().unwrap().block_on(http_err_async());
    }
}
