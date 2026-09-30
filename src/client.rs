use reqwest::Url;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::Error;

/// Default host of the forecast API.
pub const FORECAST_BASE: &str = "https://api.open-meteo.com/";
/// Default host of the historical weather (archive) API.
pub const ARCHIVE_BASE: &str = "https://archive-api.open-meteo.com/";
/// Default host of the air quality API.
pub const AIR_QUALITY_BASE: &str = "https://air-quality-api.open-meteo.com/";
/// Default host of the geocoding API.
pub const GEOCODING_BASE: &str = "https://geocoding-api.open-meteo.com/";

/// Host of the commercial forecast API.
pub const CUSTOMER_FORECAST_BASE: &str = "https://customer-api.open-meteo.com/";
/// Host of the commercial historical weather (archive) API.
pub const CUSTOMER_ARCHIVE_BASE: &str = "https://customer-archive-api.open-meteo.com/";
/// Host of the commercial air quality API.
pub const CUSTOMER_AIR_QUALITY_BASE: &str = "https://customer-air-quality-api.open-meteo.com/";
/// Host of the commercial geocoding API.
pub const CUSTOMER_GEOCODING_BASE: &str = "https://customer-geocoding-api.open-meteo.com/";

const FORECAST_PATH: &str = "v1/forecast";
const ARCHIVE_PATH: &str = "v1/archive";
const AIR_QUALITY_PATH: &str = "v1/air-quality";
const GEOCODING_PATH: &str = "v1/search";

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub(crate) type Query = Vec<(&'static str, String)>;

/// An Open-Meteo API client.
///
/// Cloning is cheap: clones share the underlying connection pool.
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    pub(crate) forecast_url: Url,
    pub(crate) archive_url: Url,
    pub(crate) air_quality_url: Url,
    pub(crate) geocoding_url: Url,
    apikey: Option<String>,
}

impl Client {
    /// A client for the free, non-commercial Open-Meteo hosts.
    ///
    /// ### Errors
    ///
    /// Returns an `Err` if the default HTTP client cannot be built.
    pub fn new() -> Result<Self, Error> {
        Self::builder().build()
    }

    /// Starts configuring a client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// The forecast endpoint this client calls.
    #[must_use]
    pub fn forecast_url(&self) -> &Url {
        &self.forecast_url
    }

    /// The archive endpoint this client calls.
    #[must_use]
    pub fn archive_url(&self) -> &Url {
        &self.archive_url
    }

    /// The air quality endpoint this client calls.
    #[must_use]
    pub fn air_quality_url(&self) -> &Url {
        &self.air_quality_url
    }

    /// The geocoding endpoint this client calls.
    #[must_use]
    pub fn geocoding_url(&self) -> &Url {
        &self.geocoding_url
    }

    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        url: &Url,
        mut query: Query,
    ) -> Result<T, Error> {
        if let Some(apikey) = &self.apikey {
            query.push(("apikey", apikey.clone()));
        }
        let response = self.http.get(url.clone()).query(&query).send().await?;
        let status = response.status();
        let body = response.bytes().await?;
        if !status.is_success() {
            return Err(refused(status.as_u16(), &body));
        }
        serde_json::from_slice(&body).map_err(|error| Error::Decode(error.to_string()))
    }
}

/// Configures a [`Client`]: the HTTP client, the API hosts and the API key.
#[derive(Debug, Default)]
#[must_use]
pub struct ClientBuilder {
    http: Option<reqwest::Client>,
    forecast_base: Option<String>,
    archive_base: Option<String>,
    air_quality_base: Option<String>,
    geocoding_base: Option<String>,
    apikey: Option<String>,
}

impl ClientBuilder {
    /// Uses this HTTP client instead of the default one.
    ///
    /// The default client is built from [`http_client_builder`].
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http = Some(client);
        self
    }

    /// Sets the forecast host, for example a stub server in tests.
    ///
    /// `v1/forecast` is joined onto it, so a path prefix is kept.
    pub fn forecast_base(mut self, base: impl Into<String>) -> Self {
        self.forecast_base = Some(base.into());
        self
    }

    /// Sets the archive host; `v1/archive` is joined onto it.
    pub fn archive_base(mut self, base: impl Into<String>) -> Self {
        self.archive_base = Some(base.into());
        self
    }

    /// Sets the air quality host; `v1/air-quality` is joined onto it.
    pub fn air_quality_base(mut self, base: impl Into<String>) -> Self {
        self.air_quality_base = Some(base.into());
        self
    }

    /// Sets the geocoding host; `v1/search` is joined onto it.
    pub fn geocoding_base(mut self, base: impl Into<String>) -> Self {
        self.geocoding_base = Some(base.into());
        self
    }

    /// Sends this `apikey` with every request.
    pub fn apikey(mut self, apikey: impl Into<String>) -> Self {
        self.apikey = Some(apikey.into());
        self
    }

    /// Uses the `customer-` prefixed commercial hosts with this API key.
    ///
    /// Hosts set explicitly, before or after, take precedence.
    pub fn commercial(mut self, apikey: impl Into<String>) -> Self {
        self.forecast_base
            .get_or_insert_with(|| CUSTOMER_FORECAST_BASE.to_owned());
        self.archive_base
            .get_or_insert_with(|| CUSTOMER_ARCHIVE_BASE.to_owned());
        self.air_quality_base
            .get_or_insert_with(|| CUSTOMER_AIR_QUALITY_BASE.to_owned());
        self.geocoding_base
            .get_or_insert_with(|| CUSTOMER_GEOCODING_BASE.to_owned());
        self.apikey(apikey)
    }

    /// Builds the client.
    ///
    /// ### Errors
    ///
    /// Returns an `Err` if a base URL is unusable or the default HTTP client cannot be built.
    pub fn build(self) -> Result<Client, Error> {
        let http = match self.http {
            Some(http) => http,
            None => default_http_client()?,
        };
        Ok(Client {
            http,
            forecast_url: endpoint(
                self.forecast_base.as_deref().unwrap_or(FORECAST_BASE),
                FORECAST_PATH,
            )?,
            archive_url: endpoint(
                self.archive_base.as_deref().unwrap_or(ARCHIVE_BASE),
                ARCHIVE_PATH,
            )?,
            air_quality_url: endpoint(
                self.air_quality_base.as_deref().unwrap_or(AIR_QUALITY_BASE),
                AIR_QUALITY_PATH,
            )?,
            geocoding_url: endpoint(
                self.geocoding_base.as_deref().unwrap_or(GEOCODING_BASE),
                GEOCODING_PATH,
            )?,
            apikey: self.apikey,
        })
    }
}

/// A reqwest builder with the TLS setup of the default client: rustls on the ring
/// provider with the platform verifier, a 10 s timeout and the crate's user agent.
///
/// Use it to customize the HTTP client without installing a process-wide rustls
/// crypto provider, then pass the result to [`ClientBuilder::http_client`].
///
/// ### Errors
///
/// Returns an `Err` if the platform certificate verifier cannot be set up.
#[cfg(not(target_arch = "wasm32"))]
pub fn http_client_builder() -> Result<reqwest::ClientBuilder, Error> {
    use rustls_platform_verifier::BuilderVerifierExt;
    use std::sync::Arc;
    use std::time::Duration;

    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut tls = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .and_then(BuilderVerifierExt::with_platform_verifier)
        .map_err(|error| Error::Tls(error.to_string()))?
        .with_no_client_auth();
    tls.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(reqwest::Client::builder()
        .tls_backend_preconfigured(tls)
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .user_agent(USER_AGENT))
}

/// A reqwest builder with the crate's user agent; the browser handles TLS.
///
/// ### Errors
///
/// Never fails on wasm32; the `Result` matches the native signature.
#[cfg(target_arch = "wasm32")]
#[allow(clippy::unnecessary_wraps)]
pub fn http_client_builder() -> Result<reqwest::ClientBuilder, Error> {
    Ok(reqwest::Client::builder().user_agent(USER_AGENT))
}

fn default_http_client() -> Result<reqwest::Client, Error> {
    Ok(http_client_builder()?.build()?)
}

fn endpoint(base: &str, path: &str) -> Result<Url, Error> {
    let unusable = || Error::BaseUrl(base.to_owned());
    let mut directory = Url::parse(base).map_err(|_| unusable())?;
    let usable = matches!(directory.scheme(), "http" | "https")
        && directory.has_host()
        && directory.query().is_none()
        && directory.fragment().is_none();
    if !usable {
        return Err(unusable());
    }
    if !directory.path().ends_with('/') {
        let path = format!("{}/", directory.path());
        directory.set_path(&path);
    }
    directory.join(path).map_err(|_| unusable())
}

#[derive(Deserialize)]
struct Refusal {
    reason: String,
}

fn refused(status: u16, body: &[u8]) -> Error {
    let reason = serde_json::from_slice::<Refusal>(body).map_or_else(
        |_| String::from_utf8_lossy(body).trim().to_owned(),
        |refusal| refusal.reason,
    );
    Error::Refused { status, reason }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_the_free_hosts() {
        let client = Client::new().expect("the default client builds");
        assert_eq!(
            client.forecast_url().as_str(),
            "https://api.open-meteo.com/v1/forecast"
        );
        assert_eq!(
            client.archive_url().as_str(),
            "https://archive-api.open-meteo.com/v1/archive"
        );
        assert_eq!(
            client.air_quality_url().as_str(),
            "https://air-quality-api.open-meteo.com/v1/air-quality"
        );
        assert_eq!(
            client.geocoding_url().as_str(),
            "https://geocoding-api.open-meteo.com/v1/search"
        );
    }

    #[test]
    fn commercial_uses_customer_hosts_unless_overridden() {
        let client = Client::builder()
            .geocoding_base("http://127.0.0.1:9")
            .commercial("secret")
            .build()
            .expect("the client builds");
        assert_eq!(
            client.forecast_url().as_str(),
            "https://customer-api.open-meteo.com/v1/forecast"
        );
        assert_eq!(
            client.geocoding_url().as_str(),
            "http://127.0.0.1:9/v1/search"
        );
        assert_eq!(client.apikey.as_deref(), Some("secret"));
    }

    #[test]
    fn base_paths_are_kept() {
        let client = Client::builder()
            .forecast_base("http://localhost:8080/proxy")
            .build()
            .expect("the client builds");
        assert_eq!(
            client.forecast_url().as_str(),
            "http://localhost:8080/proxy/v1/forecast"
        );
    }

    #[test]
    fn unusable_base_urls_are_rejected() {
        for base in [
            "not a url",
            "ftp://example.com",
            "https://example.com/?a=1",
            "https://example.com/#x",
        ] {
            let built = Client::builder().forecast_base(base).build();
            assert!(
                matches!(&built, Err(Error::BaseUrl(given)) if given == base),
                "{base}: {built:?}"
            );
        }
    }

    #[test]
    fn refusals_carry_the_reason() {
        let error = refused(
            400,
            br#"{"reason":"Latitude must be in range","error":true}"#,
        );
        assert!(
            matches!(error, Error::Refused { status: 400, reason } if reason == "Latitude must be in range")
        );
        let error = refused(502, b"Bad Gateway\n");
        assert!(matches!(error, Error::Refused { status: 502, reason } if reason == "Bad Gateway"));
    }
}
