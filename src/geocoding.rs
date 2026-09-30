use serde::Deserialize;

use crate::Error;
use crate::client::{Client, Query};

/// Query of the geocoding API.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Place name or postal code to search for. Two characters match exactly, three or more fuzzily.
    pub name: String,
    /// Language of the returned names, lower-case, for example `en` or `de`.
    pub language: Option<String>,
    /// Number of results, 1 to 100; the API default is 10.
    pub count: Option<u16>,
    /// ISO-3166-1 alpha-2 country code to restrict results to, for example `NZ`.
    pub country_code: Option<String>,
}

impl Options {
    /// Searches for `name`.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    #[must_use]
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    #[must_use]
    pub fn with_count(mut self, count: u16) -> Self {
        self.count = Some(count);
        self
    }

    #[must_use]
    pub fn with_country_code(mut self, country_code: impl Into<String>) -> Self {
        self.country_code = Some(country_code.into());
        self
    }

    fn into_query(self) -> Query {
        let mut query = vec![("name", self.name), ("format", "json".to_owned())];
        if let Some(language) = self.language {
            query.push(("language", language));
        }
        if let Some(count) = self.count {
            query.push(("count", count.to_string()));
        }
        if let Some(country_code) = self.country_code {
            query.push(("countryCode", country_code));
        }
        query
    }
}

/// Answer of the geocoding API.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GeocodingResponse {
    /// Matching places, best first; empty when nothing matches.
    #[serde(default)]
    pub results: Vec<GeocodingResult>,
    /// Server-side generation time, in milliseconds.
    #[serde(rename = "generationtime_ms")]
    pub generation_time_ms: Option<f64>,
}

/// One place found by the geocoding API, from the `GeoNames` database.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GeocodingResult {
    /// `GeoNames` id.
    pub id: i64,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    /// Elevation above mean sea level, in meters.
    pub elevation: Option<f64>,
    /// `GeoNames` feature code, for example `PPLC` for a capital.
    pub feature_code: Option<String>,
    /// ISO-3166-1 alpha-2 country code.
    pub country_code: Option<String>,
    pub country: Option<String>,
    pub country_id: Option<i64>,
    /// IANA time zone, for example `Europe/Berlin`.
    pub timezone: Option<String>,
    pub population: Option<i64>,
    #[serde(default)]
    pub postcodes: Vec<String>,
    pub admin1: Option<String>,
    pub admin2: Option<String>,
    pub admin3: Option<String>,
    pub admin4: Option<String>,
    pub admin1_id: Option<i64>,
    pub admin2_id: Option<i64>,
    pub admin3_id: Option<i64>,
    pub admin4_id: Option<i64>,
}

impl Client {
    /// Searches places by name.
    ///
    /// ### Errors
    ///
    /// Returns an `Err` if the request fails, Open-Meteo refuses it, or the answer cannot be decoded.
    pub async fn geocoding(&self, opts: Options) -> Result<GeocodingResponse, Error> {
        self.get(&self.geocoding_url, opts.into_query()).await
    }
}
