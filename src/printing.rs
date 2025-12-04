use std::collections::HashMap;

use prettytable::{Cell, Row, Table};

use crate::{
    net::c64,
    si::{format_complex_si_unitful, var_to_si_unit},
};

pub fn print_table(
    mut headers: Vec<String>,
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
