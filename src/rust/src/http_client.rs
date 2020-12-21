use bytes::{BufMut, Bytes, BytesMut};
use geozero::error::{GeozeroError, Result};
use std::cmp::max;
use std::str;

struct HttpClient {
    client: reqwest::Client,
    url: String,
    requests_ever_made: usize,
    bytes_ever_requested: usize,
}

impl HttpClient {
    fn new(url: &str) -> Self {
        HttpClient {
            client: reqwest::Client::new(),
            url: url.to_string(),
            requests_ever_made: 0,
            bytes_ever_requested: 0,
        }
    }
    async fn get(&mut self, begin: usize, length: usize) -> Result<Bytes> {
        self.requests_ever_made += 1;
        self.bytes_ever_requested += length;
        let range = format!("bytes={}-{}", begin, begin + length - 1);
        debug!(
            "request: #{}, bytes: (this_request: {}, ever: {}), Range: {}",
            self.requests_ever_made, length, self.bytes_ever_requested, range
        );
        let response = self
            .client
            .get(&self.url)
            .header("Range", range)
            .send()
            .await
            .map_err(|e| GeozeroError::HttpError(e.to_string()))?;
        if !response.status().is_success() {
            return Err(GeozeroError::HttpStatus(response.status().as_u16()));
        }
        response
            .bytes()
            .await
            .map_err(|e| GeozeroError::HttpError(e.to_string()))
    }
}

pub struct BufferedHttpClient {
    http_client: HttpClient,
    buf: BytesMut,
    /// Lower index of buffer relative to input stream
    head: usize,
}

impl BufferedHttpClient {
    pub fn new(url: &str) -> Self {
        BufferedHttpClient {
            http_client: HttpClient::new(url),
            buf: BytesMut::new(),
            head: 0,
        }
    }

    pub async fn get(&mut self, begin: usize, length: usize, min_req_size: usize) -> Result<&[u8]> {
        // If we don't already have the data, we download it
        if begin + length > self.tail() || begin < self.head {
            // Remove bytes before new begin
            if begin > self.head && begin < self.tail() {
                let _ = self.buf.split_to(begin - self.head);
                self.head = begin;
            } else if begin >= self.tail() || begin < self.head {
                self.buf.clear();
                self.head = begin;
            }

            // Read additional bytes
            let range_begin = max(begin, self.tail());
            debug!("http_client#get begin: {}, tail: {}, length: {}, range_begin: {}, min_req_size: {}", begin, self.tail(), length, range_begin, min_req_size);
            let range_length = max(length - (range_begin - begin), min_req_size);
            let bytes = self.http_client.get(range_begin, range_length).await?;
            self.buf.put(bytes);
        }

        let lower = begin - self.head;
        let upper = begin + length - self.head;
        Ok(&self.buf[lower..upper])
    }

    fn tail(&self) -> usize {
        self.head + self.buf.len()
    }
}
