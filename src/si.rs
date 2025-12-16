use std::{collections::HashMap, f64::consts::PI};

use lazy_static::lazy_static;

use crate::numbers::c64;

pub const SI_PREFIXES: &[(f64, &str)] = &[
    (1e30, "Q"),
    (1e27, "R"),
    (1e24, "Y"),
    (1e21, "Z"),
    (1e18, "E"),
    (1e15, "P"),
    (1e12, "T"),
    (1e9, "G"),
    (1e6, "M"),
    (1e3, "k"),
    (1.0, ""),
    (1e-3, "m"),
    (1e-6, "µ"),
    (1e-6, "u"), // duplicate, but that's ok
    (1e-9, "n"),
    (1e-12, "p"),
    (1e-15, "f"),
    (1e-18, "a"),
    (1e-21, "z"),
];

lazy_static! {
    static ref VAR_TO_SI_UNIT: HashMap<&'static str, &'static str> =
        [("I", "A"), ("R", "Ω"), ("V", "V"), ("C", "F"), ("P", "W")]
            .into_iter()
            .collect();
}

pub fn var_to_si_unit(var: &str) -> Option<&'static str> {
    VAR_TO_SI_UNIT.get(var).map(|v| &**v)
}

pub fn format_complex_si_unitful(z: c64, unit: &str) -> String {
    let mag = z.norm();
    let angle_deg = z.arg() * 180.0 / PI;

    let mut prefix = "";
    let mut scaled = mag;

    for (mult, pre) in SI_PREFIXES.iter() {
        let test = mag / mult;
        if test >= 1.0 && test < 1000.0 {
            scaled = test;
            prefix = pre;
            break;
        }
    }

    let decimal_places = if scaled == 0.0 {
        4
    } else {
        let digits = scaled.abs().log10().floor() as i32 + 1;
        (5 - digits).max(0) as usize
    };

    let formatted_mag = if scaled >= 1000.0 || scaled < 1e-12 {
        format!("{:.3E}", mag)
    } else {
        format!("{:.*}", decimal_places, scaled)
    };

    let formatted_angle = format!(
        "{:.1}",
        if angle_deg < 0. {
            180. + angle_deg
        } else {
            angle_deg
        }
    );

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
