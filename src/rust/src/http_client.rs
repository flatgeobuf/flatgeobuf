use bytes::{BufMut, Bytes, BytesMut};
use geozero::error::{GeozeroError, Result};
use std::cmp::max;
use std::str;

struct HttpClient {
    client: reqwest::Client,
    url: String,
}

impl HttpClient {
    fn new(url: &str) -> Self {
        HttpClient {
            client: reqwest::Client::new(),
            url: url.to_string(),
        }
    }
    async fn get(&self, begin: usize, length: usize) -> Result<Bytes> {
        let response = self
            .client
            .get(&self.url)
            .header("Range", format!("bytes={}-{}", begin, begin + length - 1))
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
        let tail = self.head + self.buf.len();
        if begin + length > tail || begin < self.head {
            // Remove bytes before new begin
            if begin > self.head && begin < tail {
                let _ = self.buf.split_to(begin - self.head);
                self.head = begin;
            } else if begin >= tail || begin < self.head {
                self.buf.clear();
                self.head = begin;
            }

            // Read additional bytes
            let range_begin = max(begin, tail);
            let range_length = max(begin + length - range_begin, min_req_size);
            let bytes = self.http_client.get(range_begin, range_length).await?;
            self.buf.put(bytes);
        }
        let lower = begin - self.head;
        let upper = begin + length - self.head;
        Ok(&self.buf[lower..upper])
    }
}
