use open_meteo_rs::forecast::{DailyParam, HourlyParam, Options};
use open_meteo_rs::jiff::civil::date;
use open_meteo_rs::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new()?;
    let opts = Options {
        hourly: vec![HourlyParam::Temperature2m],
        daily: vec![DailyParam::Temperature2mMean, DailyParam::PrecipitationSum],
        time_zone: Some("Europe/Berlin".to_owned()),
        start_date: Some(date(2023, 5, 1)),
        end_date: Some(date(2023, 5, 2)),
        ..Options::default()
    };

    let result = client.archive(opts).await?;
    if let Some(daily) = &result.daily {
        let means = daily.get(DailyParam::Temperature2mMean).unwrap_or_default();
        for (day, mean) in daily.time().iter().zip(means) {
            println!(
                "{day}: {mean:?} {}",
                daily
                    .unit(DailyParam::Temperature2mMean)
                    .unwrap_or_default()
            );
        }
    }

    Ok(())
}
