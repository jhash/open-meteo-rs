use open_meteo_rs::air_quality::{AirQualityParam, Options};
use open_meteo_rs::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new()?;
    let opts = Options {
        current: vec![AirQualityParam::EuropeanAqi, AirQualityParam::UsAqi],
        hourly: vec![AirQualityParam::Pm25, AirQualityParam::BirchPollen],
        time_zone: Some("auto".to_owned()),
        forecast_days: Some(1),
        ..Options::default()
    };

    let result = client.air_quality(opts).await?;
    if let Some(current) = &result.current {
        println!(
            "European AQI: {:?}",
            current.get(AirQualityParam::EuropeanAqi)
        );
        println!("US AQI: {:?}", current.get(AirQualityParam::UsAqi));
    }
    if let Some(hourly) = &result.hourly {
        let pm25 = hourly.get(AirQualityParam::Pm25).unwrap_or_default();
        for (time, value) in hourly.time().iter().zip(pm25) {
            println!("{time} {value:?}");
        }
    }

    Ok(())
}
