use crate::file_reader::FgbReader;
use geozero::error::Result;
use geozero::{FeatureProcessor, OpenOpts, ReadSeek, Reader, SelectOpts};

pub struct Driver<'a>(FgbReader<'a>);

impl<'a> Reader<'a> for Driver<'a> {
    fn open<R: 'a + ReadSeek>(reader: &'a mut R, _opts: &OpenOpts) -> Result<Self> {
        Ok(Driver(FgbReader::open(reader)?))
    }

    fn select(&mut self, opts: &SelectOpts) -> Result<()> {
        if let Some(bbox) = &opts.extent {
            self.0
                .select_bbox(bbox.minx, bbox.miny, bbox.maxx, bbox.maxy)?;
        } else {
            self.0.select_all()?;
        }
        Ok(())
    }

    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        self.0.process_features(processor)
    }
}

#[cfg(feature = "http")]
pub(crate) mod http {
    use crate::http_reader::HttpFgbReader;
    use async_trait::async_trait;
    use geozero::error::Result;
    use geozero::{FeatureProcessor, HttpReader, OpenOpts, SelectOpts};

    pub struct HttpDriver(HttpFgbReader);

    #[async_trait]
    impl HttpReader for HttpDriver {
        async fn open(url: String, _opts: &OpenOpts) -> Result<Self> {
            Ok(HttpDriver(HttpFgbReader::open(&url).await?))
        }

        async fn select(&mut self, opts: &SelectOpts) -> Result<()> {
            if let Some(bbox) = &opts.extent {
                self.0
                    .select_bbox(bbox.minx, bbox.miny, bbox.maxx, bbox.maxy)
                    .await?;
            } else {
                self.0.select_all().await?;
            }
            Ok(())
        }

        async fn process<P: FeatureProcessor + Send>(&mut self, processor: &mut P) -> Result<()> {
            self.0.process_features(processor).await?;
            Ok(())
        }
    }
}
