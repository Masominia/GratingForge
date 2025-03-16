//! Refractive indices with first-order dispersion around a reference wavelength.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    /// Index at the reference wavelength.
    pub n0: f64,
    /// dn / dlambda per micrometre.
    pub slope: f64,
    /// Reference wavelength, micrometres.
    pub lambda0: f64,
}

impl Material {
    pub const fn constant(n: f64) -> Material {
        Material { n0: n, slope: 0.0, lambda0: 1.55 }
    }

    pub fn index(&self, lambda: f64) -> f64 {
        self.n0 + self.slope * (lambda - self.lambda0)
    }

    /// Group index n - lambda dn/dlambda.
    pub fn group_index(&self, lambda: f64) -> f64 {
        self.index(lambda) - lambda * self.slope
    }
}

/// Crystalline silicon near 1.55 um: n = 3.476, group index about 3.61.
pub const SILICON: Material = Material { n0: 3.476, slope: -0.0865, lambda0: 1.55 };
/// Thermal oxide near 1.55 um: n = 1.444, group index about 1.463.
pub const SILICA: Material = Material { n0: 1.444, slope: -0.0120, lambda0: 1.55 };
/// Stoichiometric silicon nitride near 1.55 um.
pub const NITRIDE: Material = Material { n0: 1.996, slope: -0.0240, lambda0: 1.55 };
pub const AIR: Material = Material::constant(1.0);

/// A material by name, or a constant index if the text is a number.
pub fn by_name(text: &str) -> Result<Material, String> {
    match text {
        "si" | "silicon" => Ok(SILICON),
        "sio2" | "silica" | "oxide" => Ok(SILICA),
        "sin" | "nitride" => Ok(NITRIDE),
        "air" => Ok(AIR),
        other => other.parse::<f64>().ok().filter(|n| *n >= 1.0).map(Material::constant).ok_or_else(|| format!("'{other}' is neither a known material nor an index of at least 1")),
    }
}
