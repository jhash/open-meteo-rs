mod support;

use open_meteo_rs::air_quality::{self, AirQualityDomain, AirQualityParam};
use open_meteo_rs::forecast::{self, CellSelection, DailyParam, HourlyParam, Model};
use open_meteo_rs::geocoding;
use open_meteo_rs::jiff::Timestamp;
use open_meteo_rs::jiff::civil::{date, datetime};
use open_meteo_rs::{Error, Location};
use support::{
    AIR_QUALITY, AIR_QUALITY_PATH, ARCHIVE, ARCHIVE_PATH, EMPTY_SEARCH, SEARCH, SEARCH_PATH,
    Upstream, listed,
};

fn at(text: &str) -> Timestamp {
    text.parse().expect("a valid instant")
}

#[tokio::test]
async fn archive_reads_history_at_the_response_offset() {
    let upstream = Upstream::start().await;
    upstream.reply(ARCHIVE_PATH, 200, ARCHIVE);
    let opts = forecast::Options {
        hourly: vec![HourlyParam::Temperature2m],
        daily: vec![DailyParam::Temperature2mMean, DailyParam::PrecipitationSum],
        start_date: Some(date(2026, 3, 28)),
        end_date: Some(date(2026, 3, 29)),
        time_zone: Some("Europe/Berlin".to_owned()),
        models: vec![Model::Era5],
        ..forecast::Options::default()
    };
    let result = upstream
        .client()
        .archive(opts)
        .await
        .expect("the stub archive converts");

    let hit = upstream.only_hit(ARCHIVE_PATH);
    assert_eq!(hit["start_date"], "2026-03-28");
    assert_eq!(hit["end_date"], "2026-03-29");
    assert_eq!(hit["models"], "era5");
    assert_eq!(hit["timezone"], "Europe/Berlin");

    assert_eq!(result.meta.utc_offset_seconds, 3600);
    let hourly = result.hourly.expect("hourly values");
    assert_eq!(
        hourly.time(),
        [
            at("2026-03-29T00:00Z"),
            at("2026-03-29T01:00Z"),
            at("2026-03-29T02:00Z")
        ],
        "the fixed +01:00 applies after Berlin switches to summer time at 02:00"
    );
    let daily = result.daily.expect("daily values");
    assert_eq!(daily.time(), [date(2026, 3, 28), date(2026, 3, 29)]);
    assert_eq!(
        daily.get(DailyParam::PrecipitationSum),
        Some(&[Some(0.0), Some(1.4)][..])
    );
    assert_eq!(daily.unit(DailyParam::Temperature2mMean), Some("°C"));
}

#[tokio::test]
async fn air_quality_reads_current_and_hourly_values() {
    let upstream = Upstream::start().await;
    upstream.reply(AIR_QUALITY_PATH, 200, AIR_QUALITY);
    let opts = air_quality::Options {
        location: Location {
            lat: 52.52,
            lng: 13.41,
        },
        hourly: vec![AirQualityParam::Pm10, AirQualityParam::BirchPollen],
        current: vec![AirQualityParam::EuropeanAqi, AirQualityParam::Pm25],
        domains: Some(AirQualityDomain::CamsEurope),
        time_zone: Some("auto".to_owned()),
        forecast_days: Some(1),
        past_hours: Some(2),
        start_hour: Some(datetime(2026, 9, 30, 0, 0, 0, 0)),
        cell_selection: Some(CellSelection::Nearest),
        ..air_quality::Options::default()
    };
    let result = upstream
        .client()
        .air_quality(opts)
        .await
        .expect("the stub air quality converts");

    let hit = upstream.only_hit(AIR_QUALITY_PATH);
    assert_eq!(listed(&hit, "hourly"), ["pm10", "birch_pollen"]);
    assert_eq!(listed(&hit, "current"), ["european_aqi", "pm2_5"]);
    for (name, value) in [
        ("domains", "cams_europe"),
        ("timezone", "auto"),
        ("forecast_days", "1"),
        ("past_hours", "2"),
        ("start_hour", "2026-09-30T00:00"),
        ("cell_selection", "nearest"),
    ] {
        assert_eq!(hit.get(name).map(String::as_str), Some(value), "{name}");
    }

    let current = result.current.expect("current values");
    assert_eq!(current.time, at("2026-09-30T12:00Z"));
    assert_eq!(current.interval_seconds, Some(3600));
    assert_eq!(current.get(AirQualityParam::EuropeanAqi), Some(21.0));
    assert_eq!(current.unit(AirQualityParam::Pm25), Some("μg/m³"));

    let hourly = result.hourly.expect("hourly values");
    assert_eq!(
        hourly.time(),
        [at("2026-09-29T22:00Z"), at("2026-09-29T23:00Z")]
    );
    assert_eq!(
        hourly.get(AirQualityParam::Pm10),
        Some(&[Some(9.8), Some(10.4)][..])
    );
    assert_eq!(
        hourly.get(AirQualityParam::BirchPollen),
        Some(&[None, None][..])
    );
}

#[tokio::test]
async fn geocoding_returns_typed_places() {
    let upstream = Upstream::start().await;
    upstream.reply(SEARCH_PATH, 200, SEARCH);
    let opts = geocoding::Options::new("Auckland")
        .with_language("en")
        .with_count(10)
        .with_country_code("NZ");
    let found = upstream
        .client()
        .geocoding(opts)
        .await
        .expect("the stub search converts");

    let hit = upstream.only_hit(SEARCH_PATH);
    for (name, value) in [
        ("name", "Auckland"),
        ("language", "en"),
        ("count", "10"),
        ("countryCode", "NZ"),
        ("format", "json"),
    ] {
        assert_eq!(hit.get(name).map(String::as_str), Some(value), "{name}");
    }

    assert_eq!(found.results.len(), 4);
    let first = &found.results[0];
    assert_eq!(first.id, 2_193_733);
    assert_eq!(first.name, "Auckland");
    assert_eq!((first.latitude, first.longitude), (-36.848_53, 174.763_49));
    assert_eq!(first.timezone.as_deref(), Some("Pacific/Auckland"));
    assert_eq!(first.country_code.as_deref(), Some("NZ"));
    assert_eq!(first.population, Some(1_547_200));
    assert_eq!(first.admin1.as_deref(), Some("Auckland"));
    assert!(first.postcodes.is_empty());
    assert_eq!(found.results[2].timezone, None, "missing fields stay empty");
}

#[tokio::test]
async fn geocoding_without_results_finds_nothing() {
    let upstream = Upstream::start().await;
    upstream.reply(SEARCH_PATH, 200, EMPTY_SEARCH);
    let found = upstream
        .client()
        .geocoding(geocoding::Options::new("zzqqxx"))
        .await
        .expect("an empty answer is not an error");
    assert!(found.results.is_empty());
}

#[tokio::test]
async fn every_api_refuses_through_the_same_error() {
    let upstream = Upstream::start().await;
    for path in [ARCHIVE_PATH, AIR_QUALITY_PATH, SEARCH_PATH] {
        upstream.reply(path, 400, support::REFUSED);
    }
    let client = upstream.client();
    let results = [
        client
            .archive(forecast::Options::default())
            .await
            .map(|_| ()),
        client
            .air_quality(air_quality::Options::default())
            .await
            .map(|_| ()),
        client
            .geocoding(geocoding::Options::new("x"))
            .await
            .map(|_| ()),
    ];
    for result in results {
        assert!(
            matches!(result, Err(Error::Refused { status: 400, .. })),
            "{result:?}"
        );
    }
}

#[tokio::test]
async fn a_custom_http_client_is_used_for_requests() {
    let upstream = Upstream::start().await;
    upstream.reply(SEARCH_PATH, 200, EMPTY_SEARCH);
    let http = open_meteo_rs::http_client_builder()
        .expect("the TLS setup builds")
        .user_agent("custom-agent/1")
        .build()
        .expect("the HTTP client builds");
    let client = upstream
        .builder()
        .http_client(http)
        .build()
        .expect("the client builds");
    client
        .geocoding(geocoding::Options::new("Berlin"))
        .await
        .expect("the stub search converts");
    assert_eq!(upstream.only_hit(SEARCH_PATH)["name"], "Berlin");
}
