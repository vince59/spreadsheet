use spreadsheet::table::Table;

// write to excel
fn test_write_to_excel() -> Result<(), Box<dyn std::error::Error>>{
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

fn test_read_from_excel() -> Result<(), Box<dyn std::error::Error>> {
    let t = Table::from_excel("C:/rust/spreadsheet/data/test.xlsx", None)?;
    println!("{:?}", t);
    Ok(())
}

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
    if let Some(mut iter) = t.col_cells_mut(1) {
        for (row_idx, cell) in iter {
            cell.set_value(format!("row {row_idx}"));
            println!("  value = {:?}", cell.value());
        }
    }

    Ok(())
}

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
    if let Some(mut iter) = t.col_cells_mut(0) {
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

fn test_row_operation() -> Result<(), Box<dyn std::error::Error>> {
    let mut t = Table::from_excel("C:/rust/spreadsheet/data/test.xlsx", None)?;
    t.add_rows(2);
    for (row, col, cell) in t.iter_cells_mut() {
        cell.set_value(format!("new r{row}c{col}"));
    }
    t.remove_row(0);
    t.insert_row(3);
    for (row_idx, row) in t.iter_rows().enumerate() {
        for (col_idx, cell) in row.iter_cells() {
            println!("({row_idx}, {col_idx}) = {:?}", cell.value());
        }
    }
    Ok(())
}

fn main()  -> Result<(), Box<dyn std::error::Error>> {
    test_write_to_excel();
    test_read_from_excel();
    test_iterators();
    test_getters();
    test_row_operation();
    Ok(())
}