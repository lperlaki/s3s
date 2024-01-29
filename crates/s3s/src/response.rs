use hyper::HeaderMap;
use hyper::http::Extensions;
use hyper::http::HeaderValue;

#[non_exhaustive]
pub struct S3Response<T> {
    /// Operation output
    pub output: T,

    /// Response headers, overrides the headers in `output`.
    pub headers: HeaderMap<HeaderValue>,

    /// Response extensions.
    ///
    /// It is used to pass custom data between middlewares.
    pub extensions: Extensions,
}

impl<T: std::fmt::Debug> std::fmt::Debug for S3Response<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("S3Response");
        builder.field("output", &self.output);
        if !self.headers.is_empty() {
            builder.field("headers", &self.headers);
        }
        if !self.extensions.is_empty() {
            builder.field("extensions", &self.extensions);
        }
        builder.finish_non_exhaustive()
    }
}

impl<T> S3Response<T> {
    pub fn new(output: T) -> Self {
        Self {
            output,
            headers: HeaderMap::new(),
            extensions: Extensions::new(),
        }
    }

    pub fn with_headers(output: T, headers: HeaderMap<HeaderValue>) -> Self {
        Self {
            output,
            headers,
            extensions: Extensions::new(),
        }
    }

    pub fn map_output<U>(self, f: impl FnOnce(T) -> U) -> S3Response<U> {
        S3Response {
            output: f(self.output),
            headers: self.headers,
            extensions: self.extensions,
        }
    }
}
