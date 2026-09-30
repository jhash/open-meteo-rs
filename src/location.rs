use serde::{Deserialize, Serialize};

/// A point on the globe, in WGS84 degrees.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Location {
    /// Latitude, from -90 to 90.
    pub lat: f64,
    /// Longitude, from -180 to 180.
    pub lng: f64,
}

impl Default for Location {
    fn default() -> Self {
        Self {
            lat: 52.52,
            lng: 13.41,
        }
    }
}

impl Location {
    pub(crate) fn push_to(self, query: &mut crate::client::Query) {
        query.push(("latitude", self.lat.to_string()));
        query.push(("longitude", self.lng.to_string()));
    }
}
