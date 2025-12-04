use std::collections::HashMap;

use prettytable::{Cell, Row, Table};
use textplots::{Chart, Plot, Shape};

use crate::{
    net::c64,
    si::{format_complex_si_unitful, var_to_si_unit},
};

pub fn print_table(
    headers: Vec<String>,
    rows: Vec<(Option<String>, HashMap<String, c64>)>,
) -> String {
    let mut table = Table::new();

    let header_row = table.add_row(Row::empty());

    header_row.add_cell(Cell::new("#"));

    headers
        .clone()
        .into_iter()
        .map(|s| Cell::new(&s))
        .for_each(|c| header_row.add_cell(c));

    for (name, row) in rows {
        if row.is_empty() {
            continue;
        }

        let r = table.add_row(Row::empty());
        r.add_cell(Cell::new(name.as_ref().map(|s| s.as_str()).unwrap_or("")));

        for h in &headers {
            let value = match row.get(h) {
                Some(z) => format_complex_si_unitful(*z, var_to_si_unit(&h).unwrap_or("")),
                None => "".to_string(),
            };

            r.add_cell(Cell::new(&value));
        }
    }

    table.to_string()
}

fn continuous_segments(points: &[(f64, f64)]) -> Vec<Vec<(f64, f64)>> {
    let mut segments = Vec::new();
    let mut current_segment = Vec::new();

    for &(x, y) in points {
        if x.is_finite() && y.is_finite() {
            current_segment.push((x, y));
        } else if !current_segment.is_empty() {
            segments.push(current_segment);
            current_segment = Vec::new();
        }
    }

    if !current_segment.is_empty() {
        segments.push(current_segment);
    }

    segments
}

fn min_max(points: &[(f64, f64)]) -> Option<((f64, f64), (f64, f64))> {
    let mut min_point: Option<(f64, f64)> = None;
    let mut max_point: Option<(f64, f64)> = None;

    for &(x, y) in points {
        if !x.is_finite() || !y.is_finite() {
            continue;
        }

        min_point = Some(match min_point {
            None => (x, y),
            Some((min_x, min_y)) => (min_x.min(x), min_y.min(y)),
        });

        max_point = Some(match max_point {
            None => (x, y),
            Some((max_x, max_y)) => (max_x.max(x), max_y.max(y)),
        });
    }

    match (min_point, max_point) {
        (Some(min), Some(max)) => Some((min, max)),
        _ => None,
    }
}

pub fn print_chart(chart_name: impl ToString, points: Vec<(f64, f64)>) -> String {
    let ((x_mi, y_mi), (x_ma, y_ma)) = min_max(&points[..]).unwrap_or(((0., 0.), (0., 0.)));

    let continuous: Vec<Vec<_>> = continuous_segments(&points)
        .iter()
        .map(|v| v.iter().map(|(x, y)| (*x as f32, *y as f32)).collect())
        .collect();

    let shapes: Vec<Shape<'_>> = continuous
        .iter()
        .map(|points| Shape::Lines(&points[..]))
        .collect();

    let mut chart =
        Chart::new_with_y_range(120, 40, x_mi as f32, x_ma as f32, y_mi as f32, y_ma as f32);

    let mut c = &mut chart;
    for shape in &shapes {
        c = c.lineplot(shape);
    }

    c.axis();
    c.figures();
    format!("{}\n{}", chart_name.to_string(), c.to_string())
}
