//! Air quality data model and cleaning.
//!
//! Nothing here depends on a city: every source (RMCAB in Bogotá, OpenAQ in
//! other cities, ...) is translated into these same types. In Phase 0 the crate
//! only defines the basic types and the reading-cleaning rule; source adapters
//! and interpolation (IDW) arrive in Phase 1.
//!
//! It depends on nothing from the browser, so it is tested with a regular
//! `cargo test` and then compiled to Wasm unchanged.

/// Variables Bruma uses from each station.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variable {
    /// Fine particulate matter, in µg/m³.
    Pm25,
    /// Wind speed, in m/s.
    WindSpeed,
    /// Wind direction, in degrees from north (0–360).
    WindDirection,
}

impl Variable {
    /// Range of values accepted as real measurements.
    ///
    /// Provisional: readings outside this range are discarded. The limits will
    /// be tuned with each monitoring network's documentation and with what the
    /// real data shows.
    pub fn plausible_range(self) -> (f64, f64) {
        match self {
            Variable::Pm25 => (0.0, 500.0),
            Variable::WindSpeed => (0.0, 60.0),
            Variable::WindDirection => (0.0, 360.0),
        }
    }
}

/// A monitoring station, in geographic coordinates (WGS 84).
#[derive(Debug, Clone, PartialEq)]
pub struct Station {
    /// Identifier assigned by the monitoring network.
    pub id: String,
    /// Human-readable name, as published by the network.
    pub name: String,
    /// Longitude, in decimal degrees (east is positive).
    pub lon: f64,
    /// Latitude, in decimal degrees (north is positive).
    pub lat: f64,
}

/// Turns a raw value from a report into a usable measurement.
///
/// Returns `None` when the value is not usable:
/// - it is not a finite number,
/// - it is a network "no data" code (for example `-9999` or `-999`),
/// - it is outside the variable's plausible range.
pub fn clean_reading(variable: Variable, raw: f64) -> Option<f64> {
    if !raw.is_finite() {
        return None;
    }
    let (min, max) = variable.plausible_range();
    if raw < min || raw > max {
        return None;
    }
    Some(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_normal_pm25() {
        assert_eq!(clean_reading(Variable::Pm25, 18.4), Some(18.4));
        assert_eq!(clean_reading(Variable::Pm25, 0.0), Some(0.0));
    }

    #[test]
    fn drops_no_data_codes() {
        assert_eq!(clean_reading(Variable::Pm25, -9999.0), None);
        assert_eq!(clean_reading(Variable::Pm25, -999.0), None);
        assert_eq!(clean_reading(Variable::WindSpeed, -9999.0), None);
    }

    #[test]
    fn drops_implausible_values() {
        // Values like this show up in real data and are not valid measurements.
        assert_eq!(clean_reading(Variable::Pm25, 995.0), None);
        assert_eq!(clean_reading(Variable::WindDirection, 400.0), None);
    }

    #[test]
    fn drops_non_finite() {
        assert_eq!(clean_reading(Variable::Pm25, f64::NAN), None);
        assert_eq!(clean_reading(Variable::Pm25, f64::INFINITY), None);
    }
}
