use jiff::Timestamp;
use jiff::civil::{Date, DateTime};

use crate::client::{Client, Query};
use crate::forecast::{CellSelection, push_date, push_hour, push_value, push_variables};
use crate::response::{self, Current, Meta, Raw, Series};
use crate::{Error, location};

api_param_enum! {
    /// Air quality variables, for both `hourly` and `current`.
    AirQualityParam,
    invalid = InvalidAirQualityParam,
    {
        Pm10 => "pm10",
        Pm25 => "pm2_5",
        CarbonMonoxide => "carbon_monoxide",
        CarbonDioxide => "carbon_dioxide",
        NitrogenDioxide => "nitrogen_dioxide",
        SulphurDioxide => "sulphur_dioxide",
        Ozone => "ozone",
        AerosolOpticalDepth => "aerosol_optical_depth",
        Dust => "dust",
        UvIndex => "uv_index",
        UvIndexClearSky => "uv_index_clear_sky",
        Ammonia => "ammonia",
        Methane => "methane",
        AlderPollen => "alder_pollen",
        BirchPollen => "birch_pollen",
        GrassPollen => "grass_pollen",
        MugwortPollen => "mugwort_pollen",
        OlivePollen => "olive_pollen",
        RagweedPollen => "ragweed_pollen",
        EuropeanAqi => "european_aqi",
        EuropeanAqiPm25 => "european_aqi_pm2_5",
        EuropeanAqiPm10 => "european_aqi_pm10",
        EuropeanAqiNitrogenDioxide => "european_aqi_nitrogen_dioxide",
        EuropeanAqiOzone => "european_aqi_ozone",
        EuropeanAqiSulphurDioxide => "european_aqi_sulphur_dioxide",
        UsAqi => "us_aqi",
        UsAqiPm25 => "us_aqi_pm2_5",
        UsAqiPm10 => "us_aqi_pm10",
        UsAqiNitrogenDioxide => "us_aqi_nitrogen_dioxide",
        UsAqiOzone => "us_aqi_ozone",
        UsAqiSulphurDioxide => "us_aqi_sulphur_dioxide",
        UsAqiCarbonMonoxide => "us_aqi_carbon_monoxide",
        Formaldehyde => "formaldehyde",
        Glyoxal => "glyoxal",
        NonMethaneVolatileOrganicCompounds => "non_methane_volatile_organic_compounds",
        Pm10Wildfires => "pm10_wildfires",
        PeroxyacylNitrates => "peroxyacyl_nitrates",
        SecondaryInorganicAerosol => "secondary_inorganic_aerosol",
        ResidentialElementaryCarbon => "residential_elementary_carbon",
        TotalElementaryCarbon => "total_elementary_carbon",
        Pm25TotalOrganicMatter => "pm2_5_total_organic_matter",
        SeaSaltAerosol => "sea_salt_aerosol",
        NitrogenMonoxide => "nitrogen_monoxide",
    }
}

api_param_enum! {
    /// Air quality model domains.
    AirQualityDomain,
    invalid = InvalidAirQualityDomain,
    {
        Auto => "auto",
        CamsEurope => "cams_europe",
        CamsGlobal => "cams_global",
    }
}

/// Query of the air quality API.
#[derive(Debug, Clone, Default)]
pub struct Options {
    pub location: location::Location,
    /// Variables to request in hourly intervals.
    pub hourly: Vec<AirQualityParam>,
    /// Variables to request for current conditions.
    pub current: Vec<AirQualityParam>,
    /// `hourly` variables the typed parameters do not cover, sent after [`Self::hourly`].
    pub extra_hourly: Vec<String>,
    /// `current` variables the typed parameters do not cover, sent after [`Self::current`].
    pub extra_current: Vec<String>,
    pub domains: Option<AirQualityDomain>,
    /// IANA time zone for local times, or `auto`; the API default is `GMT`.
    pub time_zone: Option<String>,
    pub past_days: Option<u8>,
    /// 0 to 7; the API default is 5.
    pub forecast_days: Option<u8>,
    pub forecast_hours: Option<u32>,
    pub past_hours: Option<u32>,
    pub start_date: Option<Date>,
    pub end_date: Option<Date>,
    pub start_hour: Option<DateTime>,
    pub end_hour: Option<DateTime>,
    pub cell_selection: Option<CellSelection>,
}

impl Options {
    fn into_query(self) -> Query {
        let mut query = Query::new();
        self.location.push_to(&mut query);
        push_variables(&mut query, "hourly", &self.hourly, &self.extra_hourly);
        push_variables(&mut query, "current", &self.current, &self.extra_current);
        push_value(&mut query, "domains", self.domains);
        push_value(&mut query, "timezone", self.time_zone);
        push_value(&mut query, "past_days", self.past_days);
        push_value(&mut query, "forecast_days", self.forecast_days);
        push_value(&mut query, "forecast_hours", self.forecast_hours);
        push_value(&mut query, "past_hours", self.past_hours);
        push_date(&mut query, "start_date", self.start_date);
        push_date(&mut query, "end_date", self.end_date);
        push_hour(&mut query, "start_hour", self.start_hour);
        push_hour(&mut query, "end_hour", self.end_hour);
        push_value(&mut query, "cell_selection", self.cell_selection);
        query
    }
}

/// Answer of the air quality API.
#[derive(Debug, Clone, PartialEq)]
pub struct AirQualityResult {
    pub meta: Meta,
    pub current: Option<Current<AirQualityParam>>,
    pub hourly: Option<Series<AirQualityParam, Timestamp>>,
}

impl AirQualityResult {
    pub(crate) fn from_raw(raw: Raw) -> Result<Self, Error> {
        let clock = raw.clock()?;
        let meta = raw.meta();
        Ok(Self {
            meta,
            current: response::current(clock, raw.current, raw.current_units)?,
            hourly: response::series(clock, raw.hourly, raw.hourly_units)?,
        })
    }
}

impl Client {
    /// Requests air quality forecasts and current conditions.
    ///
    /// ### Errors
    ///
    /// Returns an `Err` if the request fails, Open-Meteo refuses it, or the answer cannot be decoded.
    pub async fn air_quality(&self, opts: Options) -> Result<AirQualityResult, Error> {
        let raw = self.get(&self.air_quality_url, opts.into_query()).await?;
        AirQualityResult::from_raw(raw)
    }
}
