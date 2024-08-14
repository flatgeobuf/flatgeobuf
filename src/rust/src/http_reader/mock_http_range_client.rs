use crate::{HttpFgbReader, Result};
use bytes::Bytes;
use http_range_client::AsyncHttpRangeClient;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

impl HttpFgbReader<MockHttpRangeClient> {
    /// NOTE: For debugging expediency, this test class often prefers panics over returning a result.
    pub async fn mock_from_file(
        path: &str,
    ) -> Result<(
        HttpFgbReader<MockHttpRangeClient>,
        Arc<RwLock<RequestStats>>,
    )> {
        trace!("starting: opening http reader, reading header");

        let stats = Arc::new(RwLock::new(RequestStats::new()));
        let http_client = MockHttpRangeClient::new(path, stats.clone());
        let client = http_range_client::AsyncBufferedHttpRangeClient::with(http_client, path);
        Ok((Self::_open(client).await?, stats))
    }
}

/// NOTE: For debugging expediency, this test class often prefers panics over returning a result.
pub(crate) struct MockHttpRangeClient {
    path: PathBuf,
    stats: Arc<RwLock<RequestStats>>,
}

pub(crate) struct RequestStats {
    pub request_count: u64,
    pub bytes_requested: u64,
}

impl RequestStats {
    fn new() -> Self {
        Self {
            request_count: 0,
            bytes_requested: 0,
        }
    }
}

#[async_trait::async_trait]
impl AsyncHttpRangeClient for MockHttpRangeClient {
    async fn get_range(&self, url: &str, range: &str) -> http_range_client::Result<Bytes> {
        assert_eq!(url, self.path.to_str().unwrap());

        /// This is a hack, but we need the start and length of the range
        /// since all we're given is the pre-formatted range string, we
        /// need to parse it into its components
        ///
        /// For expediency, this test code panics rather than returns a result.
        fn parse_range_header(range: &str) -> Range<u64> {
            let bytes = range.strip_prefix("bytes=").unwrap();
            let parts: Vec<&str> = bytes.split('-').collect();
            assert!(parts.len() == 2);
            let start = parts[0].parse().expect("should have valid start range");
            let end: u64 = parts[1].parse().expect("should have valid end range");
            // Range headers are *inclusive*
            start..(end + 1)
        }

        let range = parse_range_header(range);
        let request_length = range.end - range.start;

        let mut stats = self
            .stats
            .write()
            .expect("test code does not handle actual concurrency");

        stats.request_count += 1;
        stats.bytes_requested += request_length;

        let mut file_reader = BufReader::new(File::open(&self.path).unwrap());
        file_reader
            .seek(SeekFrom::Start(range.start))
            .expect("unable to seek test reader");
        let mut output = vec![0; request_length as usize];
        file_reader
            .read_exact(&mut output)
            .expect("failed to read from test reader");
        Ok(Bytes::from(output))
    }

    async fn head_response_header(
        &self,
        _url: &str,
        _header: &str,
    ) -> http_range_client::Result<Option<String>> {
        unimplemented!()
    }
}

impl MockHttpRangeClient {
    fn new(path: &str, stats: Arc<RwLock<RequestStats>>) -> Self {
        Self {
            path: path.into(),
            stats,
        }
    }
}
