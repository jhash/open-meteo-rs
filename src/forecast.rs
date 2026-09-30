use std::fmt::Display;

use jiff::Timestamp;
use jiff::civil::{Date, DateTime};

use crate::client::{Client, Query};
use crate::response::{self, Current, Meta, Raw, Series};
use crate::{Error, errors, location};

/// Elevation used for statistical downscaling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Elevation {
    /// `nan`: disables downscaling and uses the grid cell's average height.
    Nan,
    /// Meters above sea level.
    Value(f64),
}

impl Display for Elevation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nan => write!(f, "nan"),
            Self::Value(v) => write!(f, "{v}"),
        }
    }
}

impl From<Elevation> for String {
    fn from(value: Elevation) -> Self {
        value.to_string()
    }
}

impl TryFrom<&str> for Elevation {
    type Error = errors::ConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "nan" {
            return Ok(Self::Nan);
        }
        value
            .parse()
            .map(Self::Value)
            .map_err(|_| errors::ConversionError::InvalidElevation {
                name: value.to_string(),
            })
    }
}

impl From<f64> for Elevation {
    fn from(value: f64) -> Self {
        Self::Value(value)
    }
}

api_param_enum! {
    /// `temperature_unit`; the API default is Celsius.
    TemperatureUnit,
    invalid = InvalidTemperatureUnit,
    {
        Celsius => "celsius",
        Fahrenheit => "fahrenheit",
    }
}

api_param_enum! {
    /// `wind_speed_unit`; the API default is km/h.
    WindSpeedUnit,
    invalid = InvalidWindSpeedUnit,
    {
        Kmh => "kmh",
        Ms => "ms",
        Mph => "mph",
        Kn => "kn",
    }
}

api_param_enum! {
    /// `precipitation_unit`; the API default is millimeters.
    PrecipitationUnit,
    invalid = InvalidPrecipitationUnit,
    {
        Millimeters => "mm",
        Inches => "inch",
    }
}

api_param_enum! {
    /// `cell_selection`: which grid cell to prefer near the location.
    CellSelection,
    invalid = InvalidCellSelection,
    {
        Land => "land",
        Sea => "sea",
        Nearest => "nearest",
    }
}

api_param_enum! {
    /// Weather models for `models`. The prefixed names (`ncep_`, `dwd_`, `cmc_`) are the
    /// currently documented ones; the unprefixed ones are older aliases.
    Model,
    invalid = InvalidModel,
    {
        BestMatch => "best_match",
        GfsSeamless => "gfs_seamless",
        GfsGlobal => "gfs_global",
        GfsHrrr => "gfs_hrrr",
        MeteofranceSeamless => "meteofrance_seamless",
        MeteofranceArpegeSeamless => "meteofrance_arpege_seamless",
        MeteofranceArpegeWorld => "meteofrance_arpege_world",
        MeteofranceArpegeEurope => "meteofrance_arpege_europe",
        MeteofranceAromeSeamless => "meteofrance_arome_seamless",
        MeteofranceAromeFrance => "meteofrance_arome_france",
        MeteofranceAromeFranceHd => "meteofrance_arome_france_hd",
        JmaSeamless => "jma_seamless",
        JmaMsm => "jma_msm",
        JmaGsm => "jma_gsm",
        GemSeamless => "gem_seamless",
        GemGlobal => "gem_global",
        GemRegional => "gem_regional",
        GemHrdpsContinental => "gem_hrdps_continental",
        IconSeamless => "icon_seamless",
        IconGlobal => "icon_global",
        IconEu => "icon_eu",
        IconD2 => "icon_d2",
        EcmwfIfs04 => "ecmwf_ifs04",
        MetnoNordic => "metno_nordic",
        Era5Seamless => "era5_seamless",
        Era5 => "era5",
        Cerra => "cerra",
        Era5Land => "era5_land",
        EcmwfIfs => "ecmwf_ifs",
        Gwam => "gwam",
        Ewam => "ewam",
        GlofasSeamlessV3 => "glofas_seamless_v3",
        GlofasForecastV3 => "glofas_forecast_v3",
        GlofasConsolidatedV3 => "glofas_consolidated_v3",
        GlofasSeamlessV4 => "glofas_seamless_v4",
        GlofasForecastV4 => "glofas_forecast_v4",
        GlofasConsolidatedV4 => "glofas_consolidated_v4",
        Gfs025 => "gfs025",
        Gfs05 => "gfs05",
        CMCCCM2VHR4 => "CMCC_CM2_VHR4",
        FGOALSF3HHighressst => "FGOALS_f3_H_highresSST",
        FGOALSF3H => "FGOALS_f3_H",
        HiramSITHR => "HiRAM_SIT_HR",
        MRIAGCM32S => "MRI_AGCM3_2_S",
        ECEarth3pHR => "EC_Earth3P_HR",
        MPIESM12XR => "MPI_ESM1_2_XR",
        NICAM168S => "NICAM16_8S",
        CamsEurope => "cams_europe",
        CamsGlobal => "cams_global",
        Cfsv2 => "cfsv2",
        Era5Ocean => "era5_ocean",
        CmaGrapesGlobal => "cma_grapes_global",
        BomAccessGlobal => "bom_access_global",
        BomAccessGlobalEnsemble => "bom_access_global_ensemble",
        ArpaeCosmoSeamless => "arpae_cosmo_seamless",
        ArpaeCosmo2i => "arpae_cosmo_2i",
        ArpaeCosmo2iRuc => "arpae_cosmo_2i_ruc",
        ArpaeCosmo5m => "arpae_cosmo_5m",
        EcmwfIfs025 => "ecmwf_ifs025",
        EcmwfAifs025 => "ecmwf_aifs025",
        Gfs013 => "gfs013",
        GfsGraphcast025 => "gfs_graphcast025",
        EcmwfWam025 => "ecmwf_wam025",
        MeteofranceWave => "meteofrance_wave",
        MeteofranceCurrents => "meteofrance_currents",
        EcmwfWam025Ensemble => "ecmwf_wam025_ensemble",
        NcepGfswave025 => "ncep_gfswave025",
        NcepGefswave025 => "ncep_gefswave025",
        KnmiSeamless => "knmi_seamless",
        KnmiHarmonieAromeEurope => "knmi_harmonie_arome_europe",
        KnmiHarmonieAromeNetherlands => "knmi_harmonie_arome_netherlands",
        DmiSeamless => "dmi_seamless",
        DmiHarmonieAromeEurope => "dmi_harmonie_arome_europe",
        MetnoSeamless => "metno_seamless",
        Era5Ensemble => "era5_ensemble",
        EcmwfIfsAnalysis => "ecmwf_ifs_analysis",
        EcmwfIfsLongWindow => "ecmwf_ifs_long_window",
        EcmwfIfsAnalysisLongWindow => "ecmwf_ifs_analysis_long_window",
        UkmoGlobalDeterministic10km => "ukmo_global_deterministic_10km",
        UkmoUkDeterministic2km => "ukmo_uk_deterministic_2km",
        UkmoSeamless => "ukmo_seamless",
        NcepGfswave016 => "ncep_gfswave016",
        NcepNbmConus => "ncep_nbm_conus",
        UkmoGlobalEnsemble20km => "ukmo_global_ensemble_20km",
        EcmwfAifs025Single => "ecmwf_aifs025_single",
        JmaJaxaHimawari => "jma_jaxa_himawari",
        EumetsatSarah3 => "eumetsat_sarah3",
        EumetsatLsaSafMsg => "eumetsat_lsa_saf_msg",
        EumetsatLsaSafIodc => "eumetsat_lsa_saf_iodc",
        SatelliteRadiationSeamless => "satellite_radiation_seamless",
        KmaGdps => "kma_gdps",
        KmaLdps => "kma_ldps",
        KmaSeamless => "kma_seamless",
        ItaliaMeteoArpaeIcon2i => "italia_meteo_arpae_icon_2i",
        UkmoUkEnsemble2km => "ukmo_uk_ensemble_2km",
        MeteofranceAromeFranceHd15min => "meteofrance_arome_france_hd_15min",
        MeteofranceAromeFrance15min => "meteofrance_arome_france_15min",
        MeteoswissIconCh1 => "meteoswiss_icon_ch1",
        MeteoswissIconCh2 => "meteoswiss_icon_ch2",
        MeteoswissIconCh1Ensemble => "meteoswiss_icon_ch1_ensemble",
        MeteoswissIconCh2Ensemble => "meteoswiss_icon_ch2_ensemble",
        MeteoswissIconSeamless => "meteoswiss_icon_seamless",
        NcepNamConus => "ncep_nam_conus",
        IconD2Ruc => "icon_d2_ruc",
        EcmwfSeas5 => "ecmwf_seas5",
        EcmwfEc46 => "ecmwf_ec46",
        EcmwfSeasonalSeamless => "ecmwf_seasonal_seamless",
        EcmwfIfsSeamless => "ecmwf_ifs_seamless",
        JmaJaxaMtgFci => "jma_jaxa_mtg_fci",
        GemHrdpsWest => "gem_hrdps_west",
        EcmwfWam => "ecmwf_wam",
        NcepAigfs025 => "ncep_aigfs025",
        NcepAigefs025 => "ncep_aigefs025",
        NcepHgefs025EnsembleMean => "ncep_hgefs025_ensemble_mean",
        EcmwfSeasonalEnsembleMeanSeamless => "ecmwf_seasonal_ensemble_mean_seamless",
        EcmwfSeas5EnsembleMean => "ecmwf_seas5_ensemble_mean",
        EcmwfEc46EnsembleMean => "ecmwf_ec46_ensemble_mean",
        NcepAigefs025EnsembleMean => "ncep_aigefs025_ensemble_mean",
        DwdIconEpsEnsembleMeanSeamless => "dwd_icon_eps_ensemble_mean_seamless",
        DwdIconEpsEnsembleMean => "dwd_icon_eps_ensemble_mean",
        DwdIconEuEpsEnsembleMean => "dwd_icon_eu_eps_ensemble_mean",
        DwdIconD2EpsEnsembleMean => "dwd_icon_d2_eps_ensemble_mean",
        NcepGefsEnsembleMeanSeamless => "ncep_gefs_ensemble_mean_seamless",
        NcepGefs025EnsembleMean => "ncep_gefs025_ensemble_mean",
        NcepGefs05EnsembleMean => "ncep_gefs05_ensemble_mean",
        EcmwfIfs025EnsembleMean => "ecmwf_ifs025_ensemble_mean",
        EcmwfAifs025EnsembleMean => "ecmwf_aifs025_ensemble_mean",
        MeteoswissIconCh1EnsembleMean => "meteoswiss_icon_ch1_ensemble_mean",
        MeteoswissIconCh2EnsembleMean => "meteoswiss_icon_ch2_ensemble_mean",
        CmcGemGepsEnsembleMean => "cmc_gem_geps_ensemble_mean",
        UkmoGlobalEnsembleMean20km => "ukmo_global_ensemble_mean_20km",
        UkmoUkEnsembleMean2km => "ukmo_uk_ensemble_mean_2km",
        NcepGfsSeamless => "ncep_gfs_seamless",
        NcepGfsGlobal => "ncep_gfs_global",
        NcepHrrrConus => "ncep_hrrr_conus",
        DwdIconSeamless => "dwd_icon_seamless",
        DwdIconGlobal => "dwd_icon_global",
        DwdIconEu => "dwd_icon_eu",
        DwdIconD2 => "dwd_icon_d2",
        CmcGemSeamless => "cmc_gem_seamless",
        CmcGemGdps => "cmc_gem_gdps",
        CmcGemRdps => "cmc_gem_rdps",
        CmcGemHrdps => "cmc_gem_hrdps",
        CmcGemHrdpsWest => "cmc_gem_hrdps_west",
        GeosphereSeamless => "geosphere_seamless",
        GeosphereAromeAustria => "geosphere_arome_austria",
        ChmiAladinSeamless => "chmi_aladin_seamless",
        ChmiAladinCentralEurope2km => "chmi_aladin_central_europe_2km",
        ChmiAladinCz1km => "chmi_aladin_cz_1km",
    }
}

api_param_enum! {
    /// Variables for `minutely_15`: native 15-minute data in Central Europe and North
    /// America, interpolated from hourly data elsewhere.
    Minutely15Param,
    invalid = InvalidMinutely15Param,
    {
        Temperature2m => "temperature_2m",
        RelativeHumidity2m => "relative_humidity_2m",
        DewPoint2m => "dew_point_2m",
        ApparentTemperature => "apparent_temperature",
        WindSpeed10m => "wind_speed_10m",
        WindSpeed80m => "wind_speed_80m",
        WindDirection10m => "wind_direction_10m",
        WindDirection80m => "wind_direction_80m",
        WindGusts10m => "wind_gusts_10m",
        ShortwaveRadiation => "shortwave_radiation",
        DirectRadiation => "direct_radiation",
        DirectNormalIrradiance => "direct_normal_irradiance",
        DiffuseRadiation => "diffuse_radiation",
        GlobalTiltedIrradiance => "global_tilted_irradiance",
        GlobalTiltedIrradianceInstant => "global_tilted_irradiance_instant",
        SunshineDuration => "sunshine_duration",
        Precipitation => "precipitation",
        Snowfall => "snowfall",
        SnowfallHeight => "snowfall_height",
        FreezingLevelHeight => "freezing_level_height",
        Rain => "rain",
        Showers => "showers",
        Cape => "cape",
        LightningPotential => "lightning_potential",
        Visibility => "visibility",
        WeatherCode => "weather_code",
        IsDay => "is_day",
    }
}

api_param_enum! {
    /// Variables for `hourly`, for the forecast and the archive API.
    ///
    /// Pressure-level variables such as `temperature_850hPa` are not covered; read them
    /// with [`Series::get_key`].
    HourlyParam,
    invalid = InvalidHourlyParam,
    {
        Rain => "rain",
        Temperature2m => "temperature_2m",
        RelativeHumidity2m => "relative_humidity_2m",
        DewPoint2m => "dew_point_2m",
        ApparentTemperature => "apparent_temperature",
        PressureMsl => "pressure_msl",
        SurfacePressure => "surface_pressure",
        CloudCover => "cloud_cover",
        CloudCoverLow => "cloud_cover_low",
        CloudCoverMid => "cloud_cover_mid",
        CloudCoverHigh => "cloud_cover_high",
        WindSpeed10m => "wind_speed_10m",
        WindSpeed80m => "wind_speed_80m",
        WindSpeed100m => "wind_speed_100m",
        WindSpeed120m => "wind_speed_120m",
        WindSpeed180m => "wind_speed_180m",
        WindDirection10m => "wind_direction_10m",
        WindDirection80m => "wind_direction_80m",
        WindDirection100m => "wind_direction_100m",
        WindDirection120m => "wind_direction_120m",
        WindDirection180m => "wind_direction_180m",
        WindGusts10m => "wind_gusts_10m",
        Temperature80m => "temperature_80m",
        Temperature120m => "temperature_120m",
        Temperature180m => "temperature_180m",
        ShortwaveRadiation => "shortwave_radiation",
        DirectRadiation => "direct_radiation",
        DirectNormalIrradiance => "direct_normal_irradiance",
        DiffuseRadiation => "diffuse_radiation",
        GlobalTiltedIrradiance => "global_tilted_irradiance",
        TerrestrialRadiation => "terrestrial_radiation",
        ShortwaveRadiationInstant => "shortwave_radiation_instant",
        DirectRadiationInstant => "direct_radiation_instant",
        DirectNormalIrradianceInstant => "direct_normal_irradiance_instant",
        DiffuseRadiationInstant => "diffuse_radiation_instant",
        GlobalTiltedIrradianceInstant => "global_tilted_irradiance_instant",
        TerrestrialRadiationInstant => "terrestrial_radiation_instant",
        SunshineDuration => "sunshine_duration",
        VapourPressureDeficit => "vapour_pressure_deficit",
        Evapotranspiration => "evapotranspiration",
        Et0FaoEvapotranspiration => "et0_fao_evapotranspiration",
        WeatherCode => "weather_code",
        Precipitation => "precipitation",
        Showers => "showers",
        Snowfall => "snowfall",
        PrecipitationProbability => "precipitation_probability",
        SnowDepth => "snow_depth",
        SnowDepthWaterEquivalent => "snow_depth_water_equivalent",
        FreezingLevelHeight => "freezing_level_height",
        Visibility => "visibility",
        Cape => "cape",
        LiftedIndex => "lifted_index",
        ConvectiveInhibition => "convective_inhibition",
        BoundaryLayerHeight => "boundary_layer_height",
        TotalColumnIntegratedWaterVapour => "total_column_integrated_water_vapour",
        WetBulbTemperature2m => "wet_bulb_temperature_2m",
        UvIndex => "uv_index",
        UvIndexClearSky => "uv_index_clear_sky",
        IsDay => "is_day",
        Albedo => "albedo",
        SoilTemperature0cm => "soil_temperature_0cm",
        SoilTemperature6cm => "soil_temperature_6cm",
        SoilTemperature18cm => "soil_temperature_18cm",
        SoilTemperature54cm => "soil_temperature_54cm",
        SoilMoisture0To1cm => "soil_moisture_0_to_1cm",
        SoilMoisture1To3cm => "soil_moisture_1_to_3cm",
        SoilMoisture3To9cm => "soil_moisture_3_to_9cm",
        SoilMoisture9To27cm => "soil_moisture_9_to_27cm",
        SoilMoisture27To81cm => "soil_moisture_27_to_81cm",
        SoilTemperature0To10cm => "soil_temperature_0_to_10cm",
        SoilTemperature10To40cm => "soil_temperature_10_to_40cm",
        SoilTemperature40To100cm => "soil_temperature_40_to_100cm",
        SoilTemperature100To200cm => "soil_temperature_100_to_200cm",
        SoilMoisture0To10cm => "soil_moisture_0_to_10cm",
        SoilMoisture10To40cm => "soil_moisture_10_to_40cm",
        SoilMoisture40To100cm => "soil_moisture_40_to_100cm",
        SoilMoisture100To200cm => "soil_moisture_100_to_200cm",
        SoilTemperature0To7cm => "soil_temperature_0_to_7cm",
        SoilTemperature7To28cm => "soil_temperature_7_to_28cm",
        SoilTemperature28To100cm => "soil_temperature_28_to_100cm",
        SoilTemperature100To255cm => "soil_temperature_100_to_255cm",
        SoilMoisture0To7cm => "soil_moisture_0_to_7cm",
        SoilMoisture7To28cm => "soil_moisture_7_to_28cm",
        SoilMoisture28To100cm => "soil_moisture_28_to_100cm",
        SoilMoisture100To255cm => "soil_moisture_100_to_255cm",
    }
}

api_param_enum! {
    /// Variables for `daily`, for the forecast and the archive API.
    ///
    /// `sunrise` and `sunset` are instants: read them with [`Series::instants`].
    DailyParam,
    invalid = InvalidDailyParam,
    {
        WeatherCode => "weather_code",
        Temperature2mMax => "temperature_2m_max",
        Temperature2mMin => "temperature_2m_min",
        Temperature2mMean => "temperature_2m_mean",
        ApparentTemperatureMax => "apparent_temperature_max",
        ApparentTemperatureMin => "apparent_temperature_min",
        ApparentTemperatureMean => "apparent_temperature_mean",
        PrecipitationSum => "precipitation_sum",
        RainSum => "rain_sum",
        ShowersSum => "showers_sum",
        SnowfallSum => "snowfall_sum",
        SnowfallWaterEquivalentSum => "snowfall_water_equivalent_sum",
        PrecipitationHours => "precipitation_hours",
        PrecipitationProbabilityMax => "precipitation_probability_max",
        PrecipitationProbabilityMin => "precipitation_probability_min",
        PrecipitationProbabilityMean => "precipitation_probability_mean",
        Sunrise => "sunrise",
        Sunset => "sunset",
        SunshineDuration => "sunshine_duration",
        DaylightDuration => "daylight_duration",
        UvIndexMax => "uv_index_max",
        UvIndexClearSkyMax => "uv_index_clear_sky_max",
        WindSpeed10mMax => "wind_speed_10m_max",
        WindSpeed10mMean => "wind_speed_10m_mean",
        WindSpeed10mMin => "wind_speed_10m_min",
        WindGusts10mMax => "wind_gusts_10m_max",
        WindGusts10mMean => "wind_gusts_10m_mean",
        WindGusts10mMin => "wind_gusts_10m_min",
        WindDirection10mDominant => "wind_direction_10m_dominant",
        ShortwaveRadiationSum => "shortwave_radiation_sum",
        Et0FaoEvapotranspiration => "et0_fao_evapotranspiration",
        Et0FaoEvapotranspirationSum => "et0_fao_evapotranspiration_sum",
        CapeMean => "cape_mean",
        CapeMax => "cape_max",
        CapeMin => "cape_min",
        CloudCoverMean => "cloud_cover_mean",
        CloudCoverMax => "cloud_cover_max",
        CloudCoverMin => "cloud_cover_min",
        DewPoint2mMean => "dew_point_2m_mean",
        DewPoint2mMax => "dew_point_2m_max",
        DewPoint2mMin => "dew_point_2m_min",
        RelativeHumidity2mMean => "relative_humidity_2m_mean",
        RelativeHumidity2mMax => "relative_humidity_2m_max",
        RelativeHumidity2mMin => "relative_humidity_2m_min",
        PressureMslMean => "pressure_msl_mean",
        PressureMslMax => "pressure_msl_max",
        PressureMslMin => "pressure_msl_min",
        SurfacePressureMean => "surface_pressure_mean",
        SurfacePressureMax => "surface_pressure_max",
        SurfacePressureMin => "surface_pressure_min",
        VisibilityMean => "visibility_mean",
        VisibilityMax => "visibility_max",
        VisibilityMin => "visibility_min",
        WetBulbTemperature2mMean => "wet_bulb_temperature_2m_mean",
        WetBulbTemperature2mMax => "wet_bulb_temperature_2m_max",
        WetBulbTemperature2mMin => "wet_bulb_temperature_2m_min",
        VapourPressureDeficitMax => "vapour_pressure_deficit_max",
        UpdraftMax => "updraft_max",
        LeafWetnessProbabilityMean => "leaf_wetness_probability_mean",
        GrowingDegreeDaysBase0Limit50 => "growing_degree_days_base_0_limit_50",
        SoilTemperature0To7cmMean => "soil_temperature_0_to_7cm_mean",
        SoilTemperature7To28cmMean => "soil_temperature_7_to_28cm_mean",
        SoilTemperature28To100cmMean => "soil_temperature_28_to_100cm_mean",
        SoilTemperature0To100cmMean => "soil_temperature_0_to_100cm_mean",
        SoilMoisture0To7cmMean => "soil_moisture_0_to_7cm_mean",
        SoilMoisture7To28cmMean => "soil_moisture_7_to_28cm_mean",
        SoilMoisture28To100cmMean => "soil_moisture_28_to_100cm_mean",
        SoilMoisture0To100cmMean => "soil_moisture_0_to_100cm_mean",
    }
}

api_param_enum! {
    /// Variables for `current` conditions.
    CurrentParam,
    invalid = InvalidCurrentParam,
    {
        Temperature2m => "temperature_2m",
        RelativeHumidity2m => "relative_humidity_2m",
        ApparentTemperature => "apparent_temperature",
        IsDayOrNight => "is_day",
        WindSpeed10m => "wind_speed_10m",
        WindDirection10m => "wind_direction_10m",
        WindGusts10m => "wind_gusts_10m",
        Precipitation => "precipitation",
        Rain => "rain",
        Showers => "showers",
        Snowfall => "snowfall",
        WeatherCode => "weather_code",
        CloudCover => "cloud_cover",
        PressureMsl => "pressure_msl",
        SurfacePressure => "surface_pressure",
    }
}

/// Query of the forecast API, also used for the archive API.
///
/// Local times are always requested as ISO 8601 and converted to instants with the
/// response's `utc_offset_seconds`.
#[derive(Debug, Clone, Default)]
pub struct Options {
    pub location: location::Location,
    pub elevation: Option<Elevation>,
    /// Variables to request for the `minutely_15` forecast.
    pub minutely_15: Vec<Minutely15Param>,
    /// Variables to request in hourly intervals.
    pub hourly: Vec<HourlyParam>,
    /// Variables to request in daily intervals; Open-Meteo requires `time_zone` with them.
    pub daily: Vec<DailyParam>,
    /// Variables to request for current conditions.
    pub current: Vec<CurrentParam>,
    pub temperature_unit: Option<TemperatureUnit>,
    pub wind_speed_unit: Option<WindSpeedUnit>,
    pub precipitation_unit: Option<PrecipitationUnit>,
    /// IANA time zone for local times, or `auto` for the location's zone; the API default is `GMT`.
    pub time_zone: Option<String>,
    /// 0 to 92.
    pub past_days: Option<u8>,
    /// 0 to 16; the API default is 7.
    pub forecast_days: Option<u8>,
    pub forecast_hours: Option<u32>,
    pub past_hours: Option<u32>,
    pub forecast_minutely_15: Option<u32>,
    pub past_minutely_15: Option<u32>,
    /// First day to return; the archive API requires it.
    pub start_date: Option<Date>,
    /// Last day to return, inclusive; the archive API requires it.
    pub end_date: Option<Date>,
    pub start_hour: Option<DateTime>,
    pub end_hour: Option<DateTime>,
    pub start_minutely_15: Option<DateTime>,
    pub end_minutely_15: Option<DateTime>,
    /// With more than one model every response key is suffixed with the model name; read
    /// those values with [`Series::get_for_model`].
    pub models: Vec<Model>,
    pub cell_selection: Option<CellSelection>,
    /// Panel tilt for `global_tilted_irradiance`, 0 to 90 degrees.
    pub tilt: Option<f64>,
    /// Panel azimuth for `global_tilted_irradiance`, -180 to 180 degrees, 0 facing south.
    pub azimuth: Option<f64>,
}

impl Options {
    /// The query string pairs this request sends, `apikey` excluded.
    #[must_use]
    pub fn as_params(&self) -> Vec<(&'static str, String)> {
        let mut query = Query::new();
        self.location.push_to(&mut query);
        query.push(("timeformat", "iso8601".to_owned()));
        push_value(&mut query, "elevation", self.elevation);
        push_list(&mut query, "current", &self.current);
        push_list(&mut query, "minutely_15", &self.minutely_15);
        push_list(&mut query, "hourly", &self.hourly);
        push_list(&mut query, "daily", &self.daily);
        push_value(&mut query, "temperature_unit", self.temperature_unit);
        push_value(&mut query, "wind_speed_unit", self.wind_speed_unit);
        push_value(&mut query, "precipitation_unit", self.precipitation_unit);
        push_value(&mut query, "timezone", self.time_zone.as_deref());
        push_value(&mut query, "past_days", self.past_days);
        push_value(&mut query, "forecast_days", self.forecast_days);
        push_value(&mut query, "forecast_hours", self.forecast_hours);
        push_value(&mut query, "past_hours", self.past_hours);
        push_value(
            &mut query,
            "forecast_minutely_15",
            self.forecast_minutely_15,
        );
        push_value(&mut query, "past_minutely_15", self.past_minutely_15);
        push_date(&mut query, "start_date", self.start_date);
        push_date(&mut query, "end_date", self.end_date);
        push_hour(&mut query, "start_hour", self.start_hour);
        push_hour(&mut query, "end_hour", self.end_hour);
        push_hour(&mut query, "start_minutely_15", self.start_minutely_15);
        push_hour(&mut query, "end_minutely_15", self.end_minutely_15);
        push_list(&mut query, "models", &self.models);
        push_value(&mut query, "cell_selection", self.cell_selection);
        push_value(&mut query, "tilt", self.tilt);
        push_value(&mut query, "azimuth", self.azimuth);
        query
    }
}

pub(crate) fn push_value(query: &mut Query, name: &'static str, value: Option<impl Display>) {
    if let Some(value) = value {
        query.push((name, value.to_string()));
    }
}

pub(crate) fn push_list<P: AsRef<str>>(query: &mut Query, name: &'static str, params: &[P]) {
    if !params.is_empty() {
        let joined: Vec<&str> = params.iter().map(AsRef::as_ref).collect();
        query.push((name, joined.join(",")));
    }
}

pub(crate) fn push_date(query: &mut Query, name: &'static str, date: Option<Date>) {
    push_value(query, name, date.map(|date| date.strftime("%Y-%m-%d")));
}

pub(crate) fn push_hour(query: &mut Query, name: &'static str, hour: Option<DateTime>) {
    push_value(
        query,
        name,
        hour.map(|hour| hour.strftime("%Y-%m-%dT%H:%M")),
    );
}

/// Answer of the forecast or archive API.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastResult {
    pub meta: Meta,
    pub current: Option<Current<CurrentParam>>,
    pub minutely_15: Option<Series<Minutely15Param, Timestamp>>,
    pub hourly: Option<Series<HourlyParam, Timestamp>>,
    pub daily: Option<Series<DailyParam, Date>>,
}

impl ForecastResult {
    fn from_raw(raw: Raw) -> Result<Self, Error> {
        let clock = raw.clock()?;
        let meta = raw.meta();
        Ok(Self {
            meta,
            current: response::current(clock, raw.current, raw.current_units)?,
            minutely_15: response::series(clock, raw.minutely_15, raw.minutely_15_units)?,
            hourly: response::series(clock, raw.hourly, raw.hourly_units)?,
            daily: response::series(clock, raw.daily, raw.daily_units)?,
        })
    }
}

impl Client {
    /// Requests forecast data.
    ///
    /// ### Errors
    ///
    /// Returns an `Err` if the request fails, Open-Meteo refuses it, or the answer cannot be decoded.
    pub async fn forecast(&self, opts: Options) -> Result<ForecastResult, Error> {
        let raw = self.get(&self.forecast_url, opts.as_params()).await?;
        ForecastResult::from_raw(raw)
    }

    /// Requests historical weather data from the archive; set `start_date` and `end_date`.
    ///
    /// ### Errors
    ///
    /// Returns an `Err` if the request fails, Open-Meteo refuses it, or the answer cannot be decoded.
    pub async fn archive(&self, opts: Options) -> Result<ForecastResult, Error> {
        let raw = self.get(&self.archive_url, opts.as_params()).await?;
        ForecastResult::from_raw(raw)
    }
}
