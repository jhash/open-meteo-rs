/// Errors returned while building a [`Client`](crate::Client) or making a request.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A base URL is not an absolute `http` or `https` URL without query or fragment.
    #[error("invalid base URL '{0}'")]
    BaseUrl(String),

    /// The TLS configuration of the default HTTP client could not be built.
    #[error("cannot configure TLS: {0}")]
    Tls(String),

    /// The HTTP client could not be built, or the request failed in transit.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// Open-Meteo answered with a non-success status.
    #[error("Open-Meteo refused the request with status {status}: {reason}")]
    Refused {
        /// The HTTP status code.
        status: u16,
        /// The `reason` of the error body, or the raw body when it has none.
        reason: String,
    },

    /// The response body does not have the expected shape.
    #[error("cannot decode the response: {0}")]
    Decode(String),
}

/// Errors converting a string into one of the typed API parameters.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConversionError {
    #[error("Invalid elevation '{name}'")]
    InvalidElevation { name: String },

    #[error("Invalid temperature unit '{name}'")]
    InvalidTemperatureUnit { name: String },

    #[error("Invalid wind speed unit '{name}'")]
    InvalidWindSpeedUnit { name: String },

    #[error("Invalid precipitation unit '{name}'")]
    InvalidPrecipitationUnit { name: String },

    #[error("Invalid model '{name}'")]
    InvalidModel { name: String },

    #[error("Invalid cell selection '{name}'")]
    InvalidCellSelection { name: String },

    #[error("Invalid hourly param '{name}'")]
    InvalidHourlyParam { name: String },

    #[error("Invalid daily param '{name}'")]
    InvalidDailyParam { name: String },

    #[error("Invalid current param '{name}'")]
    InvalidCurrentParam { name: String },

    #[error("Invalid minutely_15 param '{name}'")]
    InvalidMinutely15Param { name: String },

    #[error("Invalid air quality param '{name}'")]
    InvalidAirQualityParam { name: String },

    #[error("Invalid air quality domain '{name}'")]
    InvalidAirQualityDomain { name: String },
}
