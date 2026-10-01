use open_meteo_rs::ConversionError;
use open_meteo_rs::air_quality::{AirQualityDomain, AirQualityParam};
use open_meteo_rs::forecast::{
    CellSelection, CurrentParam, DailyParam, Elevation, HourlyParam, Minutely15Param, Model,
    PrecipitationUnit, TemperatureUnit, WindSpeedUnit,
};

macro_rules! assert_round_trips {
    ($($enum_name:ty),+ $(,)?) => {$(
        assert_eq!(<$enum_name>::ALL.len(), <$enum_name>::VARIANTS.len());
        let mut seen = std::collections::HashSet::new();
        for (name, variant) in <$enum_name>::ALL.iter().zip(<$enum_name>::VARIANTS) {
            assert!(seen.insert(*name), "{name} is listed twice in {}", stringify!($enum_name));
            assert_eq!(variant.as_str(), *name);
            assert_eq!(variant.to_string(), *name);
            assert_eq!(<$enum_name>::try_from(*name), Ok(*variant));
            assert_eq!(name.parse::<$enum_name>(), Ok(*variant));
        }
    )+};
}

#[test]
fn every_param_round_trips_through_its_api_string() {
    assert_round_trips!(
        HourlyParam,
        DailyParam,
        CurrentParam,
        Minutely15Param,
        AirQualityParam,
        AirQualityDomain,
        Model,
        TemperatureUnit,
        WindSpeedUnit,
        PrecipitationUnit,
        CellSelection,
    );
}

#[test]
fn current_accepts_every_hourly_variable() {
    for name in HourlyParam::ALL {
        assert!(
            CurrentParam::try_from(*name).is_ok(),
            "{name} is not a current param"
        );
    }
}

#[test]
fn unknown_names_are_conversion_errors() {
    assert_eq!(
        HourlyParam::try_from("temperature_3m"),
        Err(ConversionError::InvalidHourlyParam {
            name: "temperature_3m".to_owned()
        })
    );
    assert_eq!(
        "windspeed".parse::<WindSpeedUnit>(),
        Err(ConversionError::InvalidWindSpeedUnit {
            name: "windspeed".to_owned()
        })
    );
    assert!(Model::try_from("jms_gsm").is_err());
    assert!(AirQualityParam::try_from("pm25").is_err());
}

#[test]
fn elevation_reads_nan_and_meters() {
    assert_eq!(Elevation::try_from("nan"), Ok(Elevation::Nan));
    assert_eq!(Elevation::try_from("150.9"), Ok(Elevation::Value(150.9)));
    assert!(Elevation::try_from("high").is_err());
    assert_eq!(Elevation::from(8.5).to_string(), "8.5");
    assert_eq!(Elevation::Nan.to_string(), "nan");
}

#[test]
fn documented_names_are_covered() {
    for name in [
        "is_day",
        "showers",
        "uv_index",
        "wet_bulb_temperature_2m",
        "soil_moisture_27_to_81cm",
        "terrestrial_radiation_instant",
        "wind_speed_100m",
    ] {
        assert!(HourlyParam::try_from(name).is_ok(), "hourly {name}");
    }
    for name in [
        "weather_code",
        "uv_index_max",
        "temperature_2m_mean",
        "rain_sum",
    ] {
        assert!(DailyParam::try_from(name).is_ok(), "daily {name}");
    }
    for name in ["lightning_potential", "snowfall_height", "is_day"] {
        assert!(
            Minutely15Param::try_from(name).is_ok(),
            "minutely_15 {name}"
        );
    }
    for name in [
        "ncep_gfs_seamless",
        "dwd_icon_seamless",
        "cmc_gem_seamless",
        "ecmwf_ifs025",
    ] {
        assert!(Model::try_from(name).is_ok(), "model {name}");
    }
    for name in [
        "european_aqi_pm2_5",
        "us_aqi",
        "ragweed_pollen",
        "nitrogen_monoxide",
    ] {
        assert!(
            AirQualityParam::try_from(name).is_ok(),
            "air quality {name}"
        );
    }
}
