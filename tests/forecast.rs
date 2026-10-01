mod support;

use open_meteo_rs::forecast::{
    CellSelection, CurrentParam, DailyParam, Elevation, HourlyParam, Minutely15Param, Model,
    Options, PrecipitationUnit, TemperatureUnit, WindSpeedUnit,
};
use open_meteo_rs::jiff::Timestamp;
use open_meteo_rs::jiff::civil::{date, datetime};
use open_meteo_rs::{Client, Error, Location};
use support::{
    DST_FORECAST, EXTRA_FORECAST, FORECAST_PATH, MINUTELY_FORECAST, MODELS_FORECAST, Upstream,
    listed,
};

const NZST: i32 = 12 * 3600;

fn at(text: &str) -> Timestamp {
    text.parse().expect("a valid instant")
}

fn auckland() -> Options {
    Options {
        location: Location {
            lat: -36.848_53,
            lng: 174.763_49,
        },
        current: vec![CurrentParam::Temperature2m, CurrentParam::WeatherCode],
        hourly: vec![
            HourlyParam::Temperature2m,
            HourlyParam::PrecipitationProbability,
        ],
        daily: vec![
            DailyParam::Temperature2mMax,
            DailyParam::Sunrise,
            DailyParam::Sunset,
        ],
        time_zone: Some("auto".to_owned()),
        forecast_days: Some(2),
        ..Options::default()
    }
}

#[tokio::test]
async fn converts_with_fixed_response_offset_across_dst() {
    let upstream = Upstream::start().await;
    upstream.reply(FORECAST_PATH, 200, DST_FORECAST);
    let result = upstream
        .client()
        .forecast(auckland())
        .await
        .expect("the stub forecast converts");

    assert_eq!(result.meta.timezone, "Pacific/Auckland");
    assert_eq!(result.meta.utc_offset_seconds, NZST);
    assert_eq!(result.meta.offset().seconds(), NZST);

    let hourly = result.hourly.expect("hourly values");
    assert_eq!(
        hourly.time(),
        [
            at("2026-09-26T13:00Z"),
            at("2026-09-26T14:00Z"),
            at("2026-09-26T15:00Z"),
            at("2026-09-26T16:00Z"),
        ],
        "every local time is read at the response's +12:00, also after clocks jump at 02:00"
    );

    let daily = result.daily.expect("daily values");
    assert_eq!(daily.time(), [date(2026, 9, 26), date(2026, 9, 27)]);
    assert_eq!(
        daily.instants(DailyParam::Sunrise),
        Some(&[Some(at("2026-09-25T18:05Z")), Some(at("2026-09-26T18:03Z"))][..]),
        "06:03 at +12:00 is 18:03 UTC, the true sunrise, which is 07:03 NZDT on the wall clock"
    );
    assert_eq!(
        daily.instants(DailyParam::Sunset),
        Some(&[Some(at("2026-09-26T06:25Z")), None][..])
    );
    assert_eq!(
        daily.get(DailyParam::Sunrise),
        None,
        "sunrise is not numeric"
    );

    let current = result.current.expect("current values");
    assert_eq!(current.time, at("2026-09-26T09:45Z"));
    assert_eq!(current.interval_seconds, Some(900));
}

#[tokio::test]
async fn values_and_units_are_keyed_by_the_requested_params() {
    let upstream = Upstream::start().await;
    upstream.reply(FORECAST_PATH, 200, DST_FORECAST);
    let result = upstream
        .client()
        .forecast(auckland())
        .await
        .expect("the stub forecast converts");

    let hourly = result.hourly.expect("hourly values");
    assert_eq!(hourly.len(), 4);
    assert_eq!(
        hourly.get(HourlyParam::Temperature2m),
        Some(&[Some(10.9), Some(10.3), Some(9.2), None][..])
    );
    assert_eq!(
        hourly.get(HourlyParam::PrecipitationProbability),
        Some(&[Some(40.0), Some(35.0), None, Some(10.0)][..])
    );
    assert_eq!(hourly.unit(HourlyParam::Temperature2m), Some("°C"));
    assert_eq!(hourly.get(HourlyParam::Rain), None);
    assert_eq!(
        hourly.keys().collect::<Vec<_>>(),
        ["precipitation_probability", "temperature_2m"]
    );

    let current = result.current.expect("current values");
    assert_eq!(current.get(CurrentParam::Temperature2m), Some(11.0));
    assert_eq!(current.get(CurrentParam::WeatherCode), Some(61.0));
    assert_eq!(current.get(CurrentParam::IsDayOrNight), Some(0.0));
    assert_eq!(current.unit(CurrentParam::RelativeHumidity2m), Some("%"));
    assert_eq!(current.get(CurrentParam::Rain), None);
    assert!(current.keys().all(|key| key != "time" && key != "interval"));

    let daily = result.daily.expect("daily values");
    assert_eq!(
        daily.get(DailyParam::Temperature2mMax),
        Some(&[Some(15.6), Some(16.0)][..])
    );
    assert_eq!(daily.unit(DailyParam::Sunrise), Some("iso8601"));
}

#[tokio::test]
async fn the_request_uses_documented_names_and_iso_times() {
    let upstream = Upstream::start().await;
    upstream.reply(FORECAST_PATH, 200, DST_FORECAST);
    let opts = Options {
        elevation: Some(Elevation::Nan),
        minutely_15: vec![
            Minutely15Param::Temperature2m,
            Minutely15Param::LightningPotential,
        ],
        temperature_unit: Some(TemperatureUnit::Fahrenheit),
        wind_speed_unit: Some(WindSpeedUnit::Mph),
        precipitation_unit: Some(PrecipitationUnit::Inches),
        past_days: Some(1),
        forecast_hours: Some(24),
        past_hours: Some(3),
        forecast_minutely_15: Some(96),
        past_minutely_15: Some(4),
        start_hour: Some(datetime(2026, 9, 27, 1, 0, 0, 0)),
        end_hour: Some(datetime(2026, 9, 27, 4, 0, 0, 0)),
        start_minutely_15: Some(datetime(2026, 9, 27, 1, 15, 0, 0)),
        end_minutely_15: Some(datetime(2026, 9, 27, 3, 45, 0, 0)),
        models: vec![Model::BestMatch, Model::EcmwfIfs025],
        cell_selection: Some(CellSelection::Sea),
        tilt: Some(30.0),
        azimuth: Some(-15.5),
        ..auckland()
    };
    let client = upstream
        .builder()
        .apikey("k3y")
        .build()
        .expect("the client builds");
    client
        .forecast(opts)
        .await
        .expect("the stub forecast converts");

    let hit = upstream.only_hit(FORECAST_PATH);
    for (name, value) in [
        ("latitude", "-36.84853"),
        ("longitude", "174.76349"),
        ("timeformat", "iso8601"),
        ("timezone", "auto"),
        ("elevation", "nan"),
        ("temperature_unit", "fahrenheit"),
        ("wind_speed_unit", "mph"),
        ("precipitation_unit", "inch"),
        ("past_days", "1"),
        ("forecast_days", "2"),
        ("forecast_hours", "24"),
        ("past_hours", "3"),
        ("forecast_minutely_15", "96"),
        ("past_minutely_15", "4"),
        ("start_hour", "2026-09-27T01:00"),
        ("end_hour", "2026-09-27T04:00"),
        ("start_minutely_15", "2026-09-27T01:15"),
        ("end_minutely_15", "2026-09-27T03:45"),
        ("cell_selection", "sea"),
        ("tilt", "30"),
        ("azimuth", "-15.5"),
        ("apikey", "k3y"),
    ] {
        assert_eq!(hit.get(name).map(String::as_str), Some(value), "{name}");
    }
    assert!(!hit.contains_key("windspeed_unit"), "the legacy name");
    assert_eq!(listed(&hit, "current"), ["temperature_2m", "weather_code"]);
    assert_eq!(
        listed(&hit, "minutely_15"),
        ["temperature_2m", "lightning_potential"]
    );
    assert_eq!(
        listed(&hit, "hourly"),
        ["temperature_2m", "precipitation_probability"]
    );
    assert_eq!(
        listed(&hit, "daily"),
        ["temperature_2m_max", "sunrise", "sunset"]
    );
    assert_eq!(listed(&hit, "models"), ["best_match", "ecmwf_ifs025"]);
}

#[tokio::test]
async fn unset_options_are_not_sent() {
    let upstream = Upstream::start().await;
    upstream.reply(FORECAST_PATH, 200, MINUTELY_FORECAST);
    upstream
        .client()
        .forecast(Options::default())
        .await
        .expect("the stub forecast converts");
    let hit = upstream.only_hit(FORECAST_PATH);
    let mut names: Vec<&str> = hit.keys().map(String::as_str).collect();
    names.sort_unstable();
    assert_eq!(names, ["latitude", "longitude", "timeformat"]);
}

#[tokio::test]
async fn dates_are_sent_as_calendar_days() {
    let upstream = Upstream::start().await;
    upstream.reply(FORECAST_PATH, 200, DST_FORECAST);
    let opts = Options {
        start_date: Some(date(2026, 9, 26)),
        end_date: Some(date(2026, 9, 27)),
        forecast_days: None,
        ..auckland()
    };
    upstream
        .client()
        .forecast(opts)
        .await
        .expect("the stub forecast converts");
    let hit = upstream.only_hit(FORECAST_PATH);
    assert_eq!(hit["start_date"], "2026-09-26");
    assert_eq!(hit["end_date"], "2026-09-27");
    assert!(!hit.contains_key("forecast_days"));
}

#[tokio::test]
async fn minutely_15_values_are_quarter_hour_instants() {
    let upstream = Upstream::start().await;
    upstream.reply(FORECAST_PATH, 200, MINUTELY_FORECAST);
    let opts = Options {
        minutely_15: vec![
            Minutely15Param::Temperature2m,
            Minutely15Param::Precipitation,
        ],
        ..Options::default()
    };
    let result = upstream
        .client()
        .forecast(opts)
        .await
        .expect("the stub forecast converts");
    assert!(result.hourly.is_none() && result.daily.is_none() && result.current.is_none());
    let minutely = result.minutely_15.expect("minutely_15 values");
    assert_eq!(
        minutely.time(),
        [
            at("2026-09-30T12:00Z"),
            at("2026-09-30T12:15Z"),
            at("2026-09-30T12:30Z")
        ]
    );
    assert_eq!(
        minutely.get(Minutely15Param::Precipitation),
        Some(&[Some(0.0), Some(0.01), None][..])
    );
    assert_eq!(minutely.unit(Minutely15Param::Temperature2m), Some("°F"));
}

#[tokio::test]
async fn several_models_are_read_by_model() {
    let upstream = Upstream::start().await;
    upstream.reply(FORECAST_PATH, 200, MODELS_FORECAST);
    let opts = Options {
        hourly: vec![HourlyParam::Temperature2m],
        models: vec![Model::IconSeamless, Model::GfsSeamless],
        ..Options::default()
    };
    let result = upstream
        .client()
        .forecast(opts)
        .await
        .expect("the stub forecast converts");
    let hourly = result.hourly.expect("hourly values");
    assert_eq!(hourly.get(HourlyParam::Temperature2m), None);
    assert_eq!(
        hourly.get_for_model(HourlyParam::Temperature2m, Model::IconSeamless),
        Some(&[Some(12.1), Some(11.8)][..])
    );
    assert_eq!(
        hourly.get_for_model(HourlyParam::Temperature2m, Model::GfsSeamless),
        Some(&[Some(12.6), None][..])
    );
    assert_eq!(
        hourly.unit_for_model(HourlyParam::Temperature2m, Model::GfsSeamless),
        Some("°C")
    );
}

#[tokio::test]
async fn extra_variables_are_requested_after_the_typed_ones_and_read_by_key() {
    let upstream = Upstream::start().await;
    upstream.reply(FORECAST_PATH, 200, EXTRA_FORECAST);
    let opts = Options {
        current: vec![CurrentParam::Temperature2m],
        extra_current: vec!["temperature_850hPa".to_owned()],
        extra_minutely_15: vec!["new_quarter_hour_variable".to_owned()],
        extra_hourly: vec!["geopotential_height_500hPa".to_owned()],
        extra_daily: vec!["new_daily_variable".to_owned()],
        ..Options::default()
    };
    let result = upstream
        .client()
        .forecast(opts)
        .await
        .expect("the stub forecast converts");

    let hit = upstream.only_hit(FORECAST_PATH);
    assert_eq!(
        listed(&hit, "current"),
        ["temperature_2m", "temperature_850hPa"]
    );
    assert_eq!(listed(&hit, "minutely_15"), ["new_quarter_hour_variable"]);
    assert_eq!(listed(&hit, "hourly"), ["geopotential_height_500hPa"]);
    assert_eq!(listed(&hit, "daily"), ["new_daily_variable"]);

    let current = result.current.expect("current values");
    assert_eq!(current.get(CurrentParam::Temperature2m), Some(22.5));
    assert_eq!(current.get_key("temperature_850hPa"), Some(11.5));
    assert_eq!(current.unit_key("temperature_850hPa"), Some("°C"));
    let hourly = result.hourly.expect("hourly values");
    assert_eq!(
        hourly.get_key("geopotential_height_500hPa"),
        Some(&[Some(5841.0), Some(5838.0)][..])
    );
}

#[tokio::test]
async fn refusals_are_errors_with_the_reason() {
    let upstream = Upstream::start().await;
    upstream.reply(FORECAST_PATH, 400, support::REFUSED);
    let refused = upstream.client().forecast(auckland()).await;
    assert!(
        matches!(&refused, Err(Error::Refused { status: 400, reason }) if reason.starts_with("Latitude must be")),
        "{refused:?}"
    );
}

#[tokio::test]
async fn malformed_bodies_are_decode_errors() {
    for body in [
        "{\"latitude\": 1}",
        "not json",
        r#"{"latitude":0,"longitude":0,"utc_offset_seconds":0,"timezone":"GMT","hourly":{"temperature_2m":[1.0]}}"#,
        r#"{"latitude":0,"longitude":0,"utc_offset_seconds":0,"timezone":"GMT","hourly":{"time":["2026-09-30T00:00"],"temperature_2m":[1.0,2.0]}}"#,
        r#"{"latitude":0,"longitude":0,"utc_offset_seconds":0,"timezone":"GMT","hourly":{"time":["yesterday"],"temperature_2m":[1.0]}}"#,
        r#"{"latitude":0,"longitude":0,"utc_offset_seconds":0,"timezone":"GMT","hourly":{"time":["2026-09-30T00:00"],"temperature_2m":[true]}}"#,
        r#"{"latitude":0,"longitude":0,"utc_offset_seconds":999999,"timezone":"GMT"}"#,
    ] {
        let upstream = Upstream::start().await;
        upstream.reply(FORECAST_PATH, 200, body);
        let decoded = upstream.client().forecast(Options::default()).await;
        assert!(
            matches!(decoded, Err(Error::Decode(_))),
            "{body}: {decoded:?}"
        );
    }
}

#[tokio::test]
async fn an_unreachable_host_is_an_http_error() {
    let client = Client::builder()
        .forecast_base("http://127.0.0.1:9")
        .build()
        .expect("the client builds");
    let result = client.forecast(Options::default()).await;
    assert!(matches!(result, Err(Error::Http(_))), "{result:?}");
}

#[test]
fn requests_and_errors_can_cross_threads() {
    fn send<T: Send>(_: &T) {}
    fn thread_safe<T: Send + Sync + 'static>() {}
    thread_safe::<Error>();
    thread_safe::<Client>();
    thread_safe::<open_meteo_rs::forecast::ForecastResult>();
    let client = Client::new().expect("the default client builds");
    send(&client.forecast(Options::default()));
    send(&client.archive(Options::default()));
    send(&client.air_quality(open_meteo_rs::air_quality::Options::default()));
    send(&client.geocoding(open_meteo_rs::geocoding::Options::new("Paris")));
}
