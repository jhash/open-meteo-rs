# open-meteo-rs

A Rust client for the [Open-Meteo](https://open-meteo.com/) APIs: forecast (current, 15-minutely, hourly and daily), historical weather (archive), air quality and geocoding.

- Crates.io: <https://crates.io/crates/open-meteo-rs>
- Docs.rs: <https://docs.rs/open-meteo-rs/>

## Installation

```sh
cargo add open-meteo-rs
```

The default HTTP client is [reqwest](https://crates.io/crates/reqwest) 0.13 on rustls with the ring crypto provider and the platform certificate verifier. No OpenSSL, no aws-lc, and no process-wide crypto provider has to be installed. The crate needs an async runtime only for reqwest itself (tokio on native targets) and compiles for `wasm32-unknown-unknown`, where reqwest uses the browser's `fetch`.

## Usage

```rust,no_run
use open_meteo_rs::forecast::{CurrentParam, DailyParam, HourlyParam, Minutely15Param, Options};
use open_meteo_rs::{Client, Location};

# async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
let client = Client::new()?;
let opts = Options {
    location: Location { lat: 52.52, lng: 13.41 },
    time_zone: Some("auto".to_owned()),
    forecast_days: Some(2),
    current: vec![CurrentParam::Temperature2m, CurrentParam::WeatherCode],
    minutely_15: vec![Minutely15Param::Precipitation],
    hourly: vec![HourlyParam::Temperature2m, "snowfall".try_into()?],
    daily: vec![DailyParam::Temperature2mMax, DailyParam::Sunrise],
    ..Options::default()
};

let result = client.forecast(opts).await?;

if let Some(current) = &result.current {
    println!("{:?} {:?}", current.get(CurrentParam::Temperature2m), current.unit(CurrentParam::Temperature2m));
}
if let Some(hourly) = &result.hourly {
    let temperatures = hourly.get(HourlyParam::Temperature2m).unwrap_or_default();
    for (time, temperature) in hourly.time().iter().zip(temperatures) {
        println!("{time}: {temperature:?}");
    }
}
if let Some(daily) = &result.daily {
    println!("{:?}", daily.instants(DailyParam::Sunrise));
}
# Ok(())
# }
```

`Client` is `Clone`, `Send` and `Sync`, every request future is `Send`, and every error is an [`Error`](https://docs.rs/open-meteo-rs/latest/open_meteo_rs/enum.Error.html) that is `Send + Sync + 'static`.

### Typed results

Variables are requested with the parameter enums (`HourlyParam`, `DailyParam`, `CurrentParam`, `Minutely15Param`, `AirQualityParam`), and read back with the same enums:

- `Series::get(param)` returns `Option<&[Option<f64>]>`, one entry per time step, `None` where Open-Meteo returned `null`.
- `Series::instants(param)` returns `Option<&[Option<jiff::Timestamp>]>` for time-valued variables such as daily `sunrise` and `sunset`.
- `Series::unit(param)` returns the unit Open-Meteo reported, for example `°C`.
- `Series::time()` is `&[jiff::Timestamp]` for `minutely_15` and `hourly`, and `&[jiff::civil::Date]` for `daily`.
- `Current::get(param)` returns `Option<f64>`, and `Current::time` is a `jiff::Timestamp`.
- With more than one model in `models`, Open-Meteo suffixes every key with the model name; read those with `get_for_model(param, model)`.
- Variables the enums do not cover, such as pressure-level ones or variables newer than this crate, are requested with the `extra_current`, `extra_minutely_15`, `extra_hourly` and `extra_daily` strings (`extra_current` and `extra_hourly` on air quality) and read back with `get_key("temperature_850hPa")`, `instants_key` and `unit_key`.

The enums round-trip through their API strings with `TryFrom<&str>`, `FromStr`, `Display` and `as_str()`.

### Time zones and DST

Times are requested as ISO 8601 local times and converted to instants with the response's `utc_offset_seconds`. Open-Meteo renders every local time in a response with that one offset, the one in force at request time, so the instants are correct even when a DST change falls inside the requested window, while the local clock faces in the raw response are not.

`result.meta.utc_offset_seconds` and `result.meta.timezone` are exposed so callers can compute true wall-clock times with a tz database, for example `timestamp.to_zoned(jiff::tz::TimeZone::get(&result.meta.timezone)?)` with a jiff that has a tz database enabled. This crate uses jiff without one.

### Units, models and other options

```rust,no_run
use open_meteo_rs::forecast::{
    CellSelection, Elevation, Model, Options, PrecipitationUnit, TemperatureUnit, WindSpeedUnit,
};

let opts = Options {
    elevation: Some(Elevation::Nan),
    temperature_unit: Some(TemperatureUnit::Fahrenheit),
    wind_speed_unit: Some(WindSpeedUnit::Mph),
    precipitation_unit: Some(PrecipitationUnit::Inches),
    models: vec![Model::DwdIconSeamless, Model::NcepGfsSeamless],
    cell_selection: Some(CellSelection::Land),
    past_days: Some(1),
    forecast_hours: Some(48),
    forecast_minutely_15: Some(96),
    ..Options::default()
};
```

Dates (`start_date`, `end_date`) are `jiff::civil::Date`; hours (`start_hour`, `end_hour`, `start_minutely_15`, `end_minutely_15`) are `jiff::civil::DateTime`.

### Historical weather

```rust,no_run
use open_meteo_rs::forecast::{DailyParam, Options};
use open_meteo_rs::jiff::civil::date;

# async fn run(client: open_meteo_rs::Client) -> Result<(), open_meteo_rs::Error> {
let opts = Options {
    daily: vec![DailyParam::Temperature2mMean, DailyParam::PrecipitationSum],
    time_zone: Some("Europe/Berlin".to_owned()),
    start_date: Some(date(2023, 5, 1)),
    end_date: Some(date(2023, 5, 31)),
    ..Options::default()
};
let history = client.archive(opts).await?;
# Ok(())
# }
```

### Air quality

```rust,no_run
use open_meteo_rs::air_quality::{AirQualityDomain, AirQualityParam, Options};

# async fn run(client: open_meteo_rs::Client) -> Result<(), open_meteo_rs::Error> {
let opts = Options {
    current: vec![AirQualityParam::EuropeanAqi, AirQualityParam::UsAqi],
    hourly: vec![AirQualityParam::Pm25, AirQualityParam::BirchPollen],
    domains: Some(AirQualityDomain::CamsEurope),
    ..Options::default()
};
let air = client.air_quality(opts).await?;
# Ok(())
# }
```

### Geocoding

```rust,no_run
use open_meteo_rs::geocoding::Options;

# async fn run(client: open_meteo_rs::Client) -> Result<(), open_meteo_rs::Error> {
let found = client
    .geocoding(Options::new("Auckland").with_count(5).with_language("en"))
    .await?;
for place in found.results {
    println!("{} {} {} {:?}", place.name, place.latitude, place.longitude, place.timezone);
}
# Ok(())
# }
```

Every field but `id`, `name`, `latitude` and `longitude` is optional; places missing one of those four are left out, and a search without matches returns no results rather than an error.

### Custom HTTP client, hosts and API key

```rust,no_run
# fn run() -> Result<(), open_meteo_rs::Error> {
let http = open_meteo_rs::http_client_builder()?
    .timeout(std::time::Duration::from_secs(30))
    .build()?;
let client = open_meteo_rs::Client::builder()
    .http_client(http)
    .forecast_base("http://127.0.0.1:8080")
    .build()?;

let commercial = open_meteo_rs::Client::builder().commercial("my-api-key").build()?;
# Ok(())
# }
```

`http_client_builder()` returns a reqwest builder with the default TLS setup (rustls on ring, no process-wide provider needed). Any other `reqwest::Client` works too; one built with reqwest's `rustls-no-provider` feature needs a rustls crypto provider installed first.

`forecast_base`, `archive_base`, `air_quality_base` and `geocoding_base` take a host (with an optional path prefix); `v1/forecast`, `v1/archive`, `v1/air-quality` and `v1/search` are joined onto it. `commercial` switches to the `customer-` prefixed hosts and sends `apikey` with every request.

## Attribution

Open-Meteo data is licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/); the free API is for non-commercial use. Geocoding data is based on [GeoNames](https://www.geonames.org/).

## Development

```sh
cargo test          # offline: fixtures and a local stub server
cargo clippy --all-targets -- -D warnings
cargo deny check
cargo run --example forecast   # hits the live API
```
