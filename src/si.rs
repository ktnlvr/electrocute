use std::{collections::HashMap, f64::consts::PI};

use lazy_static::lazy_static;

use crate::net::c64;

pub const SI_PREFIXES: &[(f64, &str)] = &[
    (1e12, "T"),
    (1e9, "G"),
    (1e6, "M"),
    (1e3, "k"),
    (1.0, ""),
    (1e-3, "m"),
    (1e-6, "µ"),
    (1e-9, "n"),
    (1e-12, "p"),
];

lazy_static! {
    static ref VAR_TO_SI_UNIT: HashMap<&'static str, &'static str> =
        [("I", "A"), ("R", "Ω"), ("V", "V")].into_iter().collect();
}

pub fn var_to_si_unit(var: &str) -> Option<&'static str> {
    VAR_TO_SI_UNIT.get(var).map(|v| &**v)
}

pub fn format_complex_si_unitful(z: c64, unit: &str) -> String {
    let mut mag = z.norm();
    let angle_deg = z.arg() * 180.0 / PI;

    let mut prefix = "";
    for (mult, pre) in SI_PREFIXES.iter() {
        let scaled = mag / mult;
        if scaled >= 1.0 && scaled < 1000.0 {
            mag = scaled;
            prefix = pre;
            break;
        }
    }

    let sigfigs = 4;
    let decimal_places = if mag == 0.0 {
        0
    } else {
        let digits = mag.abs().log10().floor() as i32 + 1;
        (sigfigs as i32 - digits).max(0)
    };

    let formatted_mag = format!("{:.*}", decimal_places as usize, mag);
    let formatted_angle = format!("{:.2}", angle_deg);

    format!("{}{}{} ∠{}°", formatted_mag, prefix, unit, formatted_angle)
}

pub fn format_complex_si(z: c64) -> String {
    format_complex_si_unitful(z, "")
}

pub fn parse_si_number(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let last_char = s.chars().last().unwrap();
    let (num_str, multiplier) = SI_PREFIXES
        .iter()
        .find(|(_, pre)| pre == &last_char.to_string())
        .map(|(mult, _)| (&s[..s.len() - 1], *mult))
        .unwrap_or((s, 1.0));

    num_str.parse::<f64>().ok().map(|v| v * multiplier)
}
