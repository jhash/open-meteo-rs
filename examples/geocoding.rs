use open_meteo_rs::geocoding::Options;
use open_meteo_rs::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new()?;
    let found = client
        .geocoding(Options::new("Auckland").with_count(5).with_language("en"))
        .await?;

    for place in found.results {
        println!(
            "{} ({}, {}) {} {}",
            place.name,
            place.latitude,
            place.longitude,
            place.country.unwrap_or_default(),
            place.timezone.unwrap_or_default()
        );
    }

    Ok(())
}
