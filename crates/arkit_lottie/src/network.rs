use std::sync::{Arc, OnceLock};
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, StatusCode, Url};

use crate::{LottieError, LottieNetworkSource, LottieResult};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REDIRECT_LIMIT: usize = 5;
static HTTP_CLIENT: OnceLock<LottieResult<Client>> = OnceLock::new();

/// Owns the reusable HTTP connection pool for one mounted player.
#[derive(Clone)]
pub(crate) struct LottieSourceLoader {
    client: Client,
}

impl LottieSourceLoader {
    pub(crate) fn new() -> LottieResult<Self> {
        let client = HTTP_CLIENT
            .get_or_init(build_http_client)
            .as_ref()
            .map_err(Clone::clone)?
            .clone();
        Ok(Self { client })
    }

    pub(crate) async fn load(&self, source: LottieNetworkSource) -> LottieResult<Arc<[u8]>> {
        validate_limits(&source)?;
        let url = parse_url(source.url())?;
        let headers = request_headers(&source)?;
        let mut response = self
            .client
            .get(url)
            .headers(headers)
            .timeout(source.timeout())
            .send()
            .await
            .map_err(|error| network_error("LottieSourceLoader::send", error))?;

        let status = response.status();
        if !status.is_success() {
            return Err(status_error(status));
        }

        let maximum = source.max_download_bytes();
        if response
            .content_length()
            .is_some_and(|length| length > maximum as u64)
        {
            return Err(download_too_large(maximum));
        }
        let initial_capacity = response
            .content_length()
            .and_then(|length| usize::try_from(length).ok())
            .unwrap_or(0)
            .min(maximum);
        let mut body = Vec::with_capacity(initial_capacity);
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| network_error("LottieSourceLoader::read", error))?
        {
            let next_length = body.len().checked_add(chunk.len()).ok_or_else(|| {
                LottieError::network(
                    "LottieSourceLoader::read",
                    "the downloaded response length overflowed usize",
                )
            })?;
            if next_length > maximum {
                return Err(download_too_large(maximum));
            }
            body.extend_from_slice(&chunk);
        }
        if body.is_empty() {
            return Err(LottieError::invalid_source(
                "LottieSourceLoader::read",
                "the network response body is empty",
            ));
        }
        Ok(body.into())
    }
}

fn build_http_client() -> LottieResult<Client> {
    Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(REDIRECT_LIMIT))
        .user_agent(concat!("arkit-lottie/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| network_error("LottieSourceLoader::new", error))
}

fn validate_limits(source: &LottieNetworkSource) -> LottieResult<()> {
    if source.timeout().is_zero() {
        return Err(LottieError::invalid_configuration(
            "LottieNetworkSource::timeout",
            "network timeout must be greater than zero",
        ));
    }
    if source.max_download_bytes() == 0 {
        return Err(LottieError::invalid_configuration(
            "LottieNetworkSource::max_download_bytes",
            "maximum download size must be greater than zero",
        ));
    }
    Ok(())
}

fn parse_url(value: &str) -> LottieResult<Url> {
    let url = Url::parse(value).map_err(|error| {
        LottieError::network(
            "LottieSourceLoader::parse_url",
            format!("invalid network URL: {error}"),
        )
    })?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(LottieError::network(
            "LottieSourceLoader::parse_url",
            "only http and https Lottie URLs are supported",
        ));
    }
    Ok(url)
}

fn request_headers(source: &LottieNetworkSource) -> LottieResult<HeaderMap> {
    let mut headers = HeaderMap::with_capacity(source.headers().len());
    for (name, value) in source.headers() {
        let name = HeaderName::from_bytes(name.as_bytes()).map_err(|error| {
            LottieError::network(
                "LottieSourceLoader::header_name",
                format!("invalid request header name: {error}"),
            )
        })?;
        let value = HeaderValue::from_str(value).map_err(|_| {
            LottieError::network(
                "LottieSourceLoader::header_value",
                "invalid request header value",
            )
        })?;
        headers.append(name, value);
    }
    Ok(headers)
}

fn status_error(status: StatusCode) -> LottieError {
    LottieError::network(
        "LottieSourceLoader::status",
        format!("the Lottie server returned HTTP {status}"),
    )
}

fn download_too_large(maximum: usize) -> LottieError {
    LottieError::network(
        "LottieSourceLoader::read",
        format!("the Lottie response exceeds the configured {maximum}-byte limit"),
    )
}

fn network_error(operation: &'static str, error: reqwest::Error) -> LottieError {
    let error = error.without_url();
    let message = if error.is_timeout() {
        "the Lottie request timed out".to_owned()
    } else if error.is_connect() {
        format!("could not connect to the Lottie server: {error}")
    } else {
        error.to_string()
    };
    LottieError::network(operation, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_http_urls() {
        assert!(parse_url("https://example.com/loading.json").is_ok());
        assert!(parse_url("http://127.0.0.1/loading.json").is_ok());
        assert!(parse_url("file:///data/loading.json").is_err());
        assert!(parse_url("not a url").is_err());
    }

    #[test]
    fn validates_resource_limits_before_requesting() {
        let zero_timeout = LottieNetworkSource::new("https://example.com/loading.json")
            .with_timeout(Duration::ZERO);
        assert!(validate_limits(&zero_timeout).is_err());

        let zero_size =
            LottieNetworkSource::new("https://example.com/loading.json").with_max_download_bytes(0);
        assert!(validate_limits(&zero_size).is_err());
    }

    #[test]
    fn network_source_debug_redacts_header_values() {
        let source = LottieNetworkSource::new("https://example.com/loading.json")
            .with_header("authorization", "Bearer secret");
        let debug = format!("{source:?}");
        assert!(debug.contains("header_count"));
        assert!(!debug.contains("Bearer secret"));
    }
}
