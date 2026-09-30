use open_meteo_rs::forecast::{
    CellSelection, CurrentParam, DailyParam, Elevation, HourlyParam, Minutely15Param, Options,
    PrecipitationUnit, TemperatureUnit, WindSpeedUnit,
};
use open_meteo_rs::{Client, Location};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new()?;
    let opts = Options {
        location: Location {
            lat: 40.747_89,
            lng: -73.915_6,
        },
        elevation: Some(Elevation::Nan),
        temperature_unit: Some(TemperatureUnit::Fahrenheit),
        wind_speed_unit: Some(WindSpeedUnit::Mph),
        precipitation_unit: Some(PrecipitationUnit::Inches),
        time_zone: Some("auto".to_owned()),
        forecast_days: Some(2),
        cell_selection: Some(CellSelection::Land),
        current: vec![CurrentParam::Temperature2m, "is_day".try_into()?],
        minutely_15: vec![
            Minutely15Param::Temperature2m,
            Minutely15Param::Precipitation,
        ],
        forecast_minutely_15: Some(8),
        hourly: vec![
            HourlyParam::Temperature2m,
            HourlyParam::Rain,
            "snowfall".try_into()?,
        ],
        daily: vec![
            DailyParam::Temperature2mMax,
            DailyParam::Sunrise,
            DailyParam::Sunset,
        ],
        ..Options::default()
    };

    let result = client.forecast(opts).await?;
    let offset = result.meta.offset();
    println!(
        "{} ({}s from UTC)",
        result.meta.timezone, result.meta.utc_offset_seconds
    );

    if let Some(current) = &result.current {
        println!(
            "now: {:?} {}",
            current.get(CurrentParam::Temperature2m),
            current
                .unit(CurrentParam::Temperature2m)
                .unwrap_or_default()
        );
    }

    if let Some(minutely) = &result.minutely_15 {
        let temperatures = minutely
            .get(Minutely15Param::Temperature2m)
            .unwrap_or_default();
        for (time, temperature) in minutely.time().iter().zip(temperatures) {
            println!(
                "{} {temperature:?}",
                time.to_zoned(offset.to_time_zone()).datetime()
            );
        }
    }

    if let Some(daily) = &result.daily {
        let sunrises = daily.instants(DailyParam::Sunrise).unwrap_or_default();
        for (day, sunrise) in daily.time().iter().zip(sunrises) {
            println!("{day}: sunrise at {sunrise:?}");
        }
    }

    Ok(())
}
