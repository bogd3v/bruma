//! Modelo y limpieza de los datos de calidad del aire de Bogotá.
//!
//! En la Fase 0 este crate solo define los tipos básicos y la regla de limpieza
//! de lecturas. La descarga desde la RMCAB y la interpolación (IDW) llegan en la
//! Fase 1. No depende de nada del navegador, así que se prueba con `cargo test`
//! normal y luego se compila a Wasm sin cambios.

/// Variables que Bruma usa de cada estación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variable {
    /// Material particulado fino, en µg/m³.
    Pm25,
    /// Velocidad del viento, en m/s.
    WindSpeed,
    /// Dirección del viento, en grados desde el norte (0–360).
    WindDirection,
}

impl Variable {
    /// Rango de valores que se aceptan como mediciones reales.
    ///
    /// Provisional: fuera de este rango la lectura se descarta. Los límites se
    /// ajustarán con la documentación de la Secretaría Distrital de Ambiente
    /// (pedida en el derecho de petición) y con lo que muestren los datos reales.
    pub fn plausible_range(self) -> (f64, f64) {
        match self {
            Variable::Pm25 => (0.0, 500.0),
            Variable::WindSpeed => (0.0, 60.0),
            Variable::WindDirection => (0.0, 360.0),
        }
    }
}

/// Una estación de monitoreo, en coordenadas geográficas (WGS 84).
#[derive(Debug, Clone, PartialEq)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub lon: f64,
    pub lat: f64,
}

/// Convierte un valor crudo del reporte en una medición utilizable.
///
/// Devuelve `None` cuando el valor no sirve:
/// - no es un número finito,
/// - es un código de "sin dato" de la red (por ejemplo `-9999` o `-999`),
/// - está fuera del rango plausible de la variable.
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
        // Valores como este aparecieron en datos viejos de una estación de Bogotá.
        assert_eq!(clean_reading(Variable::Pm25, 995.0), None);
        assert_eq!(clean_reading(Variable::WindDirection, 400.0), None);
    }

    #[test]
    fn drops_non_finite() {
        assert_eq!(clean_reading(Variable::Pm25, f64::NAN), None);
        assert_eq!(clean_reading(Variable::Pm25, f64::INFINITY), None);
    }
}
