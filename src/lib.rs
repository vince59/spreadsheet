pub mod table {
    use calamine::{open_workbook_auto, Reader, Data};
    use umya_spreadsheet::{new_file, writer};
    use std::error::Error;

    pub type Value = Option<String>; // original value

    // a single cell
    #[derive(Clone, Debug)]
    pub struct Cell {
        value: Value,
    }

    // a row of cells
    #[derive(Debug)]
    pub struct Row {
        cells: Vec<Cell>,
    }

    impl Row {
        /// nombre de cellules dans la ligne
        pub fn len(&self) -> usize {
            self.cells.len()
        }

        pub fn is_empty(&self) -> bool {
            self.cells.is_empty()
        }

        /// Itérateur immuable sur les cellules de la ligne :
        /// (index de colonne, &Cell)
        pub fn iter_cells(&self) -> impl Iterator<Item = (usize, &Cell)> {
            self.cells.iter().enumerate()
        }

        /// Itérateur mutable sur les cellules de la ligne :
        /// (index de colonne, &mut Cell)
        pub fn iter_cells_mut(&mut self) -> impl Iterator<Item = (usize, &mut Cell)> {
            self.cells.iter_mut().enumerate()
        }
    }

    // a table of rows
    #[derive(Debug)]
    pub struct Table {
        rows: Vec<Row>,
        nb_cols: usize,
        name: String,
    }

    // a view of a column
    #[derive(Debug)]
    pub struct Col<'a> {
        cells: Vec<&'a Cell>,
    }

    impl Cell {
        pub fn value(&self) -> &Value {
            &self.value
        }

        pub fn value_mut(&mut self) -> &mut Value {
            &mut self.value
        }

        pub fn set_value<S: Into<String>>(&mut self, v: S) {
            self.value = Some(v.into());
        }

        pub fn clear(&mut self) {
            self.value = None;
        }
    }
    impl Table {
        // create a new table with nb_rows rows and nb_cols columns
        pub fn new(nb_rows: usize, nb_cols: usize, name: String) -> Self {
            let mut rows = Vec::with_capacity(nb_rows);
            for _ in 0..nb_rows {
                let mut cells = Vec::with_capacity(nb_cols);
                for _ in 0..nb_cols {
                    cells.push(Cell { value: None });
                }
                rows.push(Row { cells });
            }
            Self { rows, nb_cols, name }
        }

        pub fn rows(&self) -> usize {
            self.rows.len()
        }

        pub fn cols(&self) -> usize {
            self.nb_cols
        }

        // Read access to a cell
        pub fn get(&self, row: usize, col: usize) -> Option<&Cell> {
            if row < self.rows() && col < self.nb_cols {
                self.rows.get(row)?.cells.get(col)
            } else {
                None
            }
        }

        // Mutable access to a cell
        pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut Cell> {
            if row < self.rows() && col < self.nb_cols {
                self.rows.get_mut(row)?.cells.get_mut(col)
            } else {
                None
            }
        }

        // Iterator over rows
        pub fn iter_rows(&self) -> impl Iterator<Item = &Row> {
            self.rows.iter()
        }

        pub fn iter_rows_mut(&mut self) -> impl Iterator<Item = &mut Row> {
            self.rows.iter_mut()
        }


        // Iterator over columns
        pub fn col(&self, col_idx: usize) -> Option<Col<'_>> {
            if col_idx >= self.nb_cols {
                return None;
            }
            let mut cells = Vec::with_capacity(self.rows());
            for row in &self.rows {
                cells.push(&row.cells[col_idx]);
            }
            Some(Col { cells })
        }

        pub fn from_excel(path: &str, sheet_name: Option<&str>) -> Result<Self, Box<dyn Error>> {
            // Open the workbook
            let mut workbook = open_workbook_auto(path)?;
            // Get the sheet name
            let sheet_name = match sheet_name {
                Some(name) => name.to_string(),
                None => workbook
                    .sheet_names()
                    .get(0)
                    .ok_or("The workbook has no sheet")?
                    .clone(),
            };

            // Get the range of cells
            let range = match workbook.worksheet_range(&sheet_name) {
                Ok(range) => range,
                Err(e) => return Err(Box::new(e)),
            };

            let mut rows = Vec::new();
            let mut max_cols = 0usize;

            for row in range.rows() {
                let mut cells = Vec::new();

                for cell in row {
                    let value: Value = match cell {
                        Data::Empty => None,
                        Data::String(s) => Some(s.clone()),
                        Data::Float(f) => Some(f.to_string()),
                        Data::Int(i) => Some(i.to_string()),
                        Data::Bool(b) => Some(b.to_string()),
                        Data::DateTime(dt) => Some(dt.to_string()),
                        _ => Some(cell.to_string()),
                    };

                    cells.push(Cell { value });
                }

                max_cols = max_cols.max(cells.len());
                rows.push(Row { cells });
            }

            Ok(Table {
                rows,
                nb_cols: max_cols,
                name : sheet_name
            })
        }
        // get the number of rows
        pub fn rows_len(&self) -> usize {
            self.rows.len()
        }

        // get the number of columns
        pub fn cols_len(&self) -> usize {
            self.nb_cols
        }

        pub fn write_tables_to_excel(
            path: &str,
            sheets: Vec<Table>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let path = std::path::Path::new(path);
            let mut book = new_file();
            for (i, sheet) in sheets.into_iter().enumerate() {
                let s = book.new_sheet(sheet.name.clone());
                for (r, row) in sheet.iter_rows().enumerate() {
                    for (c, cell) in row.cells.iter().enumerate() {
                        if let Some(value) = &cell.value {
                            book.get_sheet_mut(&(i+1)).unwrap().get_cell_mut((r as u32 +1, c as u32+1)).set_value(value);
                        }
                    }
                }
            }
            book.remove_sheet(0);
            let _ = writer::xlsx::write(&book, path);
            Ok(())
        }

        // add n rows to the table
        pub fn add_rows(&mut self, n: usize) {
            for _ in 0..n {
                // add a new row with empty cells
                let mut cells = Vec::with_capacity(self.nb_cols);

                for _ in 0..self.nb_cols {
                    cells.push(Cell { value: None });
                }

                self.rows.push(Row { cells });
            }
        }

        /// add a single row to the table
        pub fn add_row(&mut self) {
            self.add_rows(1);
        }

        // return an iterator over all cells (immutable)
        pub fn iter_cells(
            &self,
        ) -> impl Iterator<Item = (usize, usize, &Cell)> {
            self.rows
                .iter()
                .enumerate()
                .flat_map(|(row_idx, row)| {
                    row.cells
                        .iter()
                        .enumerate()
                        .map(move |(col_idx, cell)| (row_idx, col_idx, cell))
                })
        }

        // return an iterator over all cells (mutable)
        pub fn iter_cells_mut(
            &mut self,
        ) -> impl Iterator<Item = (usize, usize, &mut Cell)> {
            self.rows
                .iter_mut()
                .enumerate()
                .flat_map(|(row_idx, row)| {
                    row.cells
                        .iter_mut()
                        .enumerate()
                        .map(move |(col_idx, cell)| (row_idx, col_idx, cell))
                })
        }

        // get a row (read-only)
        pub fn get_row(&self, row_idx: usize) -> Option<&Row> {
            self.rows.get(row_idx)
        }

        // get a row (mutable)
        pub fn get_row_mut(&mut self, row_idx: usize) -> Option<&mut Row> {
            self.rows.get_mut(row_idx)
        }

    }

}