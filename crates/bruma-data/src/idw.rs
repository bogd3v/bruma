//! Inverse distance weighting (IDW): an estimate of a value at one point from
//! the readings at the stations around it.

/// A cleaned reading at a position, in WGS 84 decimal degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Longitude, in decimal degrees (east is positive).
    pub lon: f64,
    /// Latitude, in decimal degrees (north is positive).
    pub lat: f64,
    /// The cleaned reading at that position.
    pub value: f64,
}

/// Below this distance, in degrees, a sample counts as being at the point.
const COINCIDENT: f64 = 1e-9;

/// Estimates the value at (`lon`, `lat`) from `samples` by inverse distance
/// weighting.
///
/// The distance from the point to a sample is planar, in degrees:
/// `dx = (sample.lon - lon) * cos(lat in radians)`, `dy = sample.lat - lat`,
/// `d = sqrt(dx² + dy²)`. The estimate is `Σ(wᵢ · valueᵢ) / Σ wᵢ` with
/// `wᵢ = 1 / dᵢ^power`.
///
/// Cases:
/// - `samples` is empty: returns `None`.
/// - `power` is not finite or is not greater than 0: returns `None`.
/// - a sample is closer than `1e-9` degrees: returns the value of the first
///   such sample in the slice, without dividing by zero.
/// - the weighted mean is not finite, as when `dᵢ^power` underflows to 0 or
///   overflows to infinity for a large `power`: returns `None`.
/// - otherwise: returns the weighted mean above.
pub fn idw_at(lon: f64, lat: f64, samples: &[Sample], power: f64) -> Option<f64> {
    if samples.is_empty() || !power.is_finite() || power <= 0.0 {
        return None;
    }
    let cos_lat = lat.to_radians().cos();
    let mut weighted = 0.0;
    let mut total = 0.0;
    for sample in samples {
        let dx = (sample.lon - lon) * cos_lat;
        let dy = sample.lat - lat;
        let d = dx.hypot(dy);
        if d < COINCIDENT {
            return Some(sample.value);
        }
        let w = 1.0 / d.powf(power);
        weighted += w * sample.value;
        total += w;
    }
    let mean = weighted / total;
    mean.is_finite().then_some(mean)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(lon: f64, lat: f64, value: f64) -> Sample {
        Sample { lon, lat, value }
    }

    #[test]
    fn returns_none_without_samples() {
        assert_eq!(idw_at(1.0, 2.0, &[], 2.0), None);
    }

    #[test]
    fn returns_none_for_a_power_that_is_not_positive() {
        let samples = [at(1.0, 2.0, 10.0)];
        assert_eq!(idw_at(1.5, 2.5, &samples, 0.0), None);
        assert_eq!(idw_at(1.5, 2.5, &samples, -1.0), None);
        assert_eq!(idw_at(1.5, 2.5, &samples, f64::NAN), None);
        assert_eq!(idw_at(1.5, 2.5, &samples, f64::INFINITY), None);
    }

    #[test]
    fn returns_the_sample_value_at_its_own_position() {
        let samples = [at(3.0, 4.0, 10.0), at(5.0, 4.0, 30.0), at(3.0, 4.0, 99.0)];
        // The first sample at the point wins, and nothing divides by zero.
        assert_eq!(idw_at(3.0, 4.0, &samples, 2.0), Some(10.0));
        // Inside the 1e-9 degree threshold still counts as the same point.
        assert_eq!(idw_at(3.0 + 1e-10, 4.0, &samples, 2.0), Some(10.0));
    }

    #[test]
    fn returns_none_when_the_weights_underflow() {
        // 0.01^1000 underflows to 0, so every weight is infinite: inf / inf.
        let samples = [at(0.01, 0.0, 10.0), at(-0.01, 0.0, 30.0)];
        assert_eq!(idw_at(0.0, 0.0, &samples, 1000.0), None);
    }

    #[test]
    fn returns_none_when_the_weights_overflow() {
        // 100^1000 overflows to infinity, so every weight is 0: 0 / 0.
        let samples = [at(100.0, 0.0, 10.0), at(-100.0, 0.0, 30.0)];
        assert_eq!(idw_at(0.0, 0.0, &samples, 1000.0), None);
    }

    #[test]
    fn equidistant_samples_give_their_mean() {
        let samples = [at(1.0, 0.0, 10.0), at(-1.0, 0.0, 30.0)];
        let got = idw_at(0.0, 0.0, &samples, 2.0).expect("an estimate");
        assert!((got - 20.0).abs() < 1e-12, "got {got}");
    }

    #[test]
    fn a_nearer_sample_weighs_more() {
        let samples = [at(1.0, 0.0, 10.0), at(3.0, 0.0, 30.0)];
        // Weights 1 and 1/9 with power 2: (10 + 30/9) / (1 + 1/9) = 12.
        let got = idw_at(0.0, 0.0, &samples, 2.0).expect("an estimate");
        assert!((got - 12.0).abs() < 1e-12, "got {got}");
        assert!(got < 20.0);
    }

    #[test]
    fn longitude_is_scaled_by_the_cosine_of_the_latitude() {
        // At 60° north a degree of longitude is half a degree of latitude, so
        // the sample one degree east is as near as the one half a degree north.
        let samples = [at(1.0, 60.0, 10.0), at(0.0, 60.5, 30.0)];
        let got = idw_at(0.0, 60.0, &samples, 2.0).expect("an estimate");
        assert!((got - 20.0).abs() < 1e-9, "got {got}");
    }
}
