use std::collections::BTreeMap;
use std::fmt::Display;
use std::marker::PhantomData;

use jiff::Timestamp;
use jiff::civil::{Date, DateTime};
use jiff::tz::Offset;
use serde::Deserialize;
use serde_json::Value;

use crate::Error;

/// Response metadata shared by the forecast, archive and air quality APIs.
#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
    /// Latitude of the grid cell the data comes from.
    pub latitude: f64,
    /// Longitude of the grid cell the data comes from.
    pub longitude: f64,
    /// Elevation used for statistical downscaling, in meters.
    pub elevation: Option<f64>,
    /// Server-side generation time, in milliseconds.
    pub generation_time_ms: Option<f64>,
    /// The one offset Open-Meteo used to render every local time in the response.
    ///
    /// It is the offset in force at request time, so local times after a DST change
    /// inside the requested window are off by the DST shift. Instants in this crate are
    /// derived with it and are correct; use [`Meta::timezone`] with a tz database for
    /// true wall-clock offsets.
    pub utc_offset_seconds: i32,
    /// The IANA time zone of the response, for example `Pacific/Auckland` or `GMT`.
    pub timezone: String,
    /// The abbreviation of the zone at request time, for example `NZST`.
    pub timezone_abbreviation: Option<String>,
}

impl Meta {
    /// [`Meta::utc_offset_seconds`] as a [`jiff::tz::Offset`].
    #[must_use]
    pub fn offset(&self) -> Offset {
        Offset::from_seconds(self.utc_offset_seconds).unwrap_or(Offset::UTC)
    }
}

/// Values of one time series section (`minutely_15`, `hourly` or `daily`).
///
/// `T` is [`Timestamp`] for sub-daily sections and [`Date`] for daily ones. Values are
/// looked up by the typed parameter `P` they were requested with; each value vector has
/// one entry per time step, `None` where Open-Meteo returned `null`.
#[derive(Debug, Clone, PartialEq)]
pub struct Series<P, T> {
    time: Vec<T>,
    values: BTreeMap<String, Vec<Option<f64>>>,
    instants: BTreeMap<String, Vec<Option<Timestamp>>>,
    units: BTreeMap<String, String>,
    param: PhantomData<fn() -> P>,
}

impl<P: AsRef<str> + Display, T> Series<P, T> {
    /// The time step of each row.
    #[must_use]
    pub fn time(&self) -> &[T] {
        &self.time
    }

    /// The number of time steps.
    #[must_use]
    pub fn len(&self) -> usize {
        self.time.len()
    }

    /// Whether the section has no time steps.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.time.is_empty()
    }

    /// Numeric values of `param`, or `None` if the response has no such numeric series.
    #[must_use]
    pub fn get(&self, param: P) -> Option<&[Option<f64>]> {
        self.get_key(param.as_ref())
    }

    /// Numeric values of `param` for one of several requested models.
    ///
    /// Open-Meteo suffixes every key with the model name when more than one model is requested.
    #[must_use]
    pub fn get_for_model(&self, param: P, model: crate::forecast::Model) -> Option<&[Option<f64>]> {
        self.get_key(&model_key(&param, model))
    }

    /// Instant values of `param`, such as daily `sunrise` and `sunset`.
    #[must_use]
    pub fn instants(&self, param: P) -> Option<&[Option<Timestamp>]> {
        self.instants_key(param.as_ref())
    }

    /// Instant values of `param` for one of several requested models.
    #[must_use]
    pub fn instants_for_model(
        &self,
        param: P,
        model: crate::forecast::Model,
    ) -> Option<&[Option<Timestamp>]> {
        self.instants_key(&model_key(&param, model))
    }

    /// The unit Open-Meteo reported for `param`, for example `°C`.
    #[must_use]
    pub fn unit(&self, param: P) -> Option<&str> {
        self.unit_key(param.as_ref())
    }

    /// The unit of `param` for one of several requested models.
    #[must_use]
    pub fn unit_for_model(&self, param: P, model: crate::forecast::Model) -> Option<&str> {
        self.unit_key(&model_key(&param, model))
    }

    /// Numeric values under a raw response key, for keys the typed parameters do not cover.
    #[must_use]
    pub fn get_key(&self, key: &str) -> Option<&[Option<f64>]> {
        self.values.get(key).map(Vec::as_slice)
    }

    /// Instant values under a raw response key.
    #[must_use]
    pub fn instants_key(&self, key: &str) -> Option<&[Option<Timestamp>]> {
        self.instants.get(key).map(Vec::as_slice)
    }

    /// The unit under a raw response key.
    #[must_use]
    pub fn unit_key(&self, key: &str) -> Option<&str> {
        self.units.get(key).map(String::as_str)
    }

    /// Every value key in the response, numeric and instant, in sorted order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        let mut keys: Vec<&str> = self
            .values
            .keys()
            .chain(self.instants.keys())
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        keys.into_iter()
    }
}

/// Values of the `current` section.
#[derive(Debug, Clone, PartialEq)]
pub struct Current<P> {
    /// The instant the values describe.
    pub time: Timestamp,
    /// The length of the interval the values aggregate, in seconds.
    pub interval_seconds: Option<i64>,
    values: BTreeMap<String, Option<f64>>,
    units: BTreeMap<String, String>,
    param: PhantomData<fn() -> P>,
}

impl<P: AsRef<str>> Current<P> {
    /// The value of `param`, or `None` if it is missing or `null`.
    #[must_use]
    pub fn get(&self, param: P) -> Option<f64> {
        self.get_key(param.as_ref())
    }

    /// The unit Open-Meteo reported for `param`.
    #[must_use]
    pub fn unit(&self, param: P) -> Option<&str> {
        self.unit_key(param.as_ref())
    }

    /// The value under a raw response key.
    #[must_use]
    pub fn get_key(&self, key: &str) -> Option<f64> {
        self.values.get(key).copied().flatten()
    }

    /// The unit under a raw response key.
    #[must_use]
    pub fn unit_key(&self, key: &str) -> Option<&str> {
        self.units.get(key).map(String::as_str)
    }

    /// Every value key in the response, in sorted order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(String::as_str)
    }
}

fn model_key(param: &impl Display, model: crate::forecast::Model) -> String {
    format!("{param}_{model}")
}

type Section = BTreeMap<String, Value>;
type Units = BTreeMap<String, String>;

#[derive(Deserialize)]
pub(crate) struct Raw {
    latitude: f64,
    longitude: f64,
    elevation: Option<f64>,
    generationtime_ms: Option<f64>,
    utc_offset_seconds: i32,
    timezone: String,
    timezone_abbreviation: Option<String>,
    pub(crate) current: Option<Section>,
    pub(crate) current_units: Option<Units>,
    pub(crate) minutely_15: Option<Section>,
    pub(crate) minutely_15_units: Option<Units>,
    pub(crate) hourly: Option<Section>,
    pub(crate) hourly_units: Option<Units>,
    pub(crate) daily: Option<Section>,
    pub(crate) daily_units: Option<Units>,
}

impl Raw {
    pub(crate) fn meta(&self) -> Meta {
        Meta {
            latitude: self.latitude,
            longitude: self.longitude,
            elevation: self.elevation,
            generation_time_ms: self.generationtime_ms,
            utc_offset_seconds: self.utc_offset_seconds,
            timezone: self.timezone.clone(),
            timezone_abbreviation: self.timezone_abbreviation.clone(),
        }
    }

    pub(crate) fn clock(&self) -> Result<Clock, Error> {
        Offset::from_seconds(self.utc_offset_seconds)
            .map(Clock)
            .map_err(|error| Error::Decode(format!("utc_offset_seconds: {error}")))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Clock(Offset);

impl Clock {
    pub(crate) fn instant(self, local: DateTime) -> Result<Timestamp, Error> {
        self.0
            .to_timestamp(local)
            .map_err(|error| Error::Decode(format!("time {local}: {error}")))
    }

    fn parse(self, key: &str, text: &str) -> Result<Timestamp, Error> {
        let local: DateTime = text.parse().map_err(|error| {
            Error::Decode(format!("{key}: '{text}' is not a local time: {error}"))
        })?;
        self.instant(local)
    }
}

pub(crate) trait Step: Sized {
    fn parse(clock: Clock, text: &str) -> Result<Self, Error>;
}

impl Step for Timestamp {
    fn parse(clock: Clock, text: &str) -> Result<Self, Error> {
        clock.parse("time", text)
    }
}

impl Step for Date {
    fn parse(_: Clock, text: &str) -> Result<Self, Error> {
        text.parse()
            .map_err(|error| Error::Decode(format!("time: '{text}' is not a date: {error}")))
    }
}

pub(crate) fn series<P, T: Step>(
    clock: Clock,
    section: Option<Section>,
    units: Option<Units>,
) -> Result<Option<Series<P, T>>, Error> {
    let Some(mut section) = section else {
        return Ok(None);
    };
    let time = match section.remove("time") {
        Some(Value::Array(steps)) => steps
            .iter()
            .map(|step| match step {
                Value::String(text) => T::parse(clock, text),
                other => Err(Error::Decode(format!(
                    "time: expected a string, got {other}"
                ))),
            })
            .collect::<Result<Vec<T>, Error>>()?,
        _ => return Err(Error::Decode("a time series has no time array".to_owned())),
    };
    let mut values = BTreeMap::new();
    let mut instants = BTreeMap::new();
    for (key, column) in section {
        let Value::Array(column) = column else {
            return Err(Error::Decode(format!("{key}: expected an array")));
        };
        if column.len() != time.len() {
            return Err(Error::Decode(format!(
                "{key}: {} values for {} time steps",
                column.len(),
                time.len()
            )));
        }
        if column.iter().any(Value::is_string) {
            let parsed = column
                .iter()
                .map(|value| match value {
                    Value::Null => Ok(None),
                    Value::String(text) => clock.parse(&key, text).map(Some),
                    other => Err(Error::Decode(format!(
                        "{key}: expected a local time, got {other}"
                    ))),
                })
                .collect::<Result<_, _>>()?;
            instants.insert(key, parsed);
        } else {
            let parsed = column
                .iter()
                .map(|value| number(&key, value))
                .collect::<Result<_, _>>()?;
            values.insert(key, parsed);
        }
    }
    Ok(Some(Series {
        time,
        values,
        instants,
        units: without_time(units),
        param: PhantomData,
    }))
}

pub(crate) fn current<P>(
    clock: Clock,
    section: Option<Section>,
    units: Option<Units>,
) -> Result<Option<Current<P>>, Error> {
    let Some(mut section) = section else {
        return Ok(None);
    };
    let time = match section.remove("time") {
        Some(Value::String(text)) => clock.parse("time", &text)?,
        _ => return Err(Error::Decode("current has no time".to_owned())),
    };
    let interval_seconds =
        match section.remove("interval") {
            None | Some(Value::Null) => None,
            Some(value) => Some(value.as_i64().ok_or_else(|| {
                Error::Decode(format!("interval: expected an integer, got {value}"))
            })?),
        };
    let values = section
        .iter()
        .map(|(key, value)| Ok((key.clone(), number(key, value)?)))
        .collect::<Result<_, Error>>()?;
    let mut units = without_time(units);
    units.remove("interval");
    Ok(Some(Current {
        time,
        interval_seconds,
        values,
        units,
        param: PhantomData,
    }))
}

fn number(key: &str, value: &Value) -> Result<Option<f64>, Error> {
    match value {
        Value::Null => Ok(None),
        Value::Number(number) => number
            .as_f64()
            .map(Some)
            .ok_or_else(|| Error::Decode(format!("{key}: {number} is not a finite number"))),
        other => Err(Error::Decode(format!(
            "{key}: expected a number, got {other}"
        ))),
    }
}

fn without_time(units: Option<Units>) -> Units {
    let mut units = units.unwrap_or_default();
    units.remove("time");
    units
}
