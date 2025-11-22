use spreadsheet::table::{Table, Value};

// write to excel
#[allow(dead_code)]
fn test_write_to_excel() -> Result<(), Box<dyn std::error::Error>> {
    let mut sheets = Vec::new();
    for i in 0..2 {
        let mut t = Table::new(3, 4, format!("Tableau {}", i));
        t.add_rows(2);
        for (row, col, cell) in t.iter_cells_mut() {
            cell.set_value(format!("T {i} r{row}c{col}"));
        }
        sheets.push(t);
    }
    Table::write_tables_to_excel("C:/rust/spreadsheet/data/test.xlsx", sheets)
}

#[allow(dead_code)]
fn test_read_from_excel() -> Result<(), Box<dyn std::error::Error>> {
    let t = Table::from_excel("C:/rust/spreadsheet/data/test.xlsx", None)?;
    println!("{:?}", t);
    Ok(())
}

#[allow(dead_code)]
fn test_iterators() -> Result<(), Box<dyn std::error::Error>> {
    let mut t = Table::from_excel("C:/rust/spreadsheet/data/test.xlsx", None)?;

    //--------------------- Rows

    // iterate on all cells one by one
    for (row, col, cell) in t.iter_cells() {
        println!("R{}C{} -> {}", row, col, cell.value().as_ref().unwrap());
    }
    // iterate on all cells in mutable mode
    for (row, col, cell) in t.iter_cells_mut() {
        cell.set_value(format!("R{}C{} new value!", row, col));
        println!("R{}C{} -> {}", row, col, cell.value().as_ref().unwrap());
    }
    // iterate on all rows and then on all cells
    for (row_idx, row) in t.iter_rows().enumerate() {
        for (col_idx, cell) in row.iter_cells() {
            println!("({row_idx}, {col_idx}) = {:?}", cell.value());
        }
    }
    // iterate on all rows and then on all cells (mutable)
    for (row_idx, row) in t.iter_rows_mut().enumerate() {
        for (col_idx, cell) in row.iter_cells_mut() {
            cell.set_value(format!("R{}C{} new value2 !", row_idx, col_idx));
            println!("({row_idx}, {col_idx}) = {:?}", cell.value());
        }
    }

    // --------------------- Columns

    // Iterate over all columns
    for col in t.iter_cols() {
        println!("Column index: {}", col.index());
        // Iterate over cells of this column
        for cell in col.iter_cells() {
            println!("  value = {:?}", cell.value());
        }
    }

    // Iterate over cells of a column in mutable mode
    if let Some(iter) = t.col_cells_mut(1) {
        for (row_idx, cell) in iter {
            cell.set_value(format!("row {row_idx}"));
            println!("  value = {:?}", cell.value());
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn test_getters() -> Result<(), Box<dyn std::error::Error>> {
    let mut t = Table::from_excel("C:/rust/spreadsheet/data/test.xlsx", None)?;

    // Get a row
    if let Some(row) = t.get_row(2) {
        for (col_idx, cell) in row.iter_cells() {
            println!("(2, {col_idx}) = {:?}", cell.value());
        }
    }

    // Write in a row
    if let Some(row) = t.get_row_mut(1) {
        for (col_idx, cell) in row.iter_cells_mut() {
            cell.set_value(format!("L1C{col_idx}"));
            println!("(2, {col_idx}) = {:?}", cell.value());
        }
    }

    // get a column
    if let Some(col0) = t.get_col(0) {
        println!("--- Column {} ---", col0.index());
        for (row_idx, cell) in col0.iter_cells().enumerate() {
            println!("row {row_idx} -> {:?}", cell.value());
        }
    }

    // Write in a column
    if let Some(iter) = t.col_cells_mut(0) {
        for (row_idx, cell) in iter {
            cell.set_value(format!("C0-R{row_idx}"));
            println!("row {row_idx} -> {:?}", cell.value());
        }
    }

    // get a cell
    if let Some(cell) = t.get(1, 2) {
        println!("Cell (1, 2) = {:?}", cell.value());
    }

    Ok(())
}
#[allow(dead_code)]
fn test_row_operation() -> Result<(), Box<dyn std::error::Error>> {
    let mut t = Table::from_excel("C:/rust/spreadsheet/data/test.xlsx", None)?;
    t.add_rows(2);
    for (row, col, cell) in t.iter_cells_mut() {
        cell.set_value(format!("new r{row}c{col}"));
    }
    t.remove_row(0)?;
    t.insert_row(3)?;
    for (row_idx, row) in t.iter_rows().enumerate() {
        for (col_idx, cell) in row.iter_cells() {
            println!("({row_idx}, {col_idx}) = {:?}", cell.value());
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn test_dupplicates() -> Result<(), Box<dyn std::error::Error>> {
    let mut t = Table::from_excel("C:/rust/spreadsheet/data/test2.xlsx", None)?;
    let col_idx = t.add_col()?;

    for row_idx in 0..t.get_nb_rows() {
        let value = if row_idx == 0 {
            "Key".to_string()
        } else {
            let v1 = t.get_cell_text(row_idx, col_idx - 3);
            let v2 = t.get_cell_text(row_idx, col_idx - 2);
            format!("{}_{}", v1, v2)
        };

        if let Some(cell) = t.get_mut(row_idx, col_idx) {
            cell.set_value(value);
        }
    }
    let dupplicates = t.find_duplicates_in_column(col_idx)?;
    let col_idx = t.add_col()?;
    dupplicates.iter().for_each(|dupp| {
        if let Some(cell) = t.get_mut(dupp.row_idx, col_idx) {
            if dupp.row_idx == 0 {
                cell.set_value("Nb doublons".to_string());
            } else {
                cell.set_value(dupp.occurrence.to_string());
            }
        }
    });

    t.to_excel("C:/rust/spreadsheet/data/result.xlsx")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //test_write_to_excel();
    //test_read_from_excel();
    //test_iterators();
    //test_getters();
    //test_row_operation();
    //test_dupplicates();
    //test_dupplicates();

    let mut tables = Table::from_excel_all_sheets("C:/rust/spreadsheet/data/annuaire.xlsx")?;
    let mut structures = tables.remove(0);
    let mut services = tables.remove(0);

    structures.first_line_as_header()?;
    services.first_line_as_header()?;
    let mut structure_header =vec! [
        "Identifiant",
        "RaisonSociale",
        "GestionEngagement",
        "GestionService",
        "Adresse",
        "ComplementAdresse1",
        "ComplementAdresse2",
        "CodePostal",
        "Ville",
        "NumTelephone",
        "Courriel",
    ];
    let cols = structures.get_cols_indices_by_titles(&structure_header)?;
    structures.keep_cols(&cols)?;
    structure_header.push("Code Chorus");
    structure_header.push("Type");
    structure_header.push("Nom du service");
    let mut customers = Table::from_column_titles(&structure_header, "Liste pour Sage");

    for (_, row) in structures.iter_rows().enumerate() {
        let i = customers.add_row();
        // Initialize with a dummy value, will be overwritten if col_idx==1 exists
        let mut customer_code = &Value::default();
        for (col_idx, cell) in row.iter_cells() {
            customers.set_cell_value(i,col_idx,cell.value().clone())?;
            if col_idx==1 {
                customer_code=cell.value();
            }
        }
        customers.set_cell_value_by_title(i,"Type",Some("Maison mère".to_string()))?;
        let srv =services.find_rows_matching_value(0, customer_code)?;
        for row_idx in srv {
            let srv_row = services.get_row(row_idx).unwrap();
            let i=customers.add_row();
            for (col_idx, cell) in srv_row.iter_cells() {
                let col = match col_idx {
                    2 => 11,
                    3 => 13,
                    _ => col_idx,
                };
                customers.set_cell_value(i,col,cell.value().clone())?;
            }
            customers.set_cell_value_by_title(i,"Type",Some("Filiale".to_string()))?;
        }
    }

    customers.insert_header_as_first_row()?;
    customers.to_excel("C:/rust/spreadsheet/data/customers.xlsx")?;
    Ok(())
}
