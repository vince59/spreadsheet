pub mod table {
    use calamine::{open_workbook_auto, Data, Reader};
    use std::{error::Error};
    use std::collections::{HashMap, HashSet};
    use umya_spreadsheet::{new_file, writer};

    // -------------------------------------------------------------------------
    // Basic types
    // -------------------------------------------------------------------------

    pub type Value = Option<String>; // original value

    // Search mode
    pub enum MatchMode {
        Exact,
        Contains,
    }

    #[derive(Debug)]
    pub struct MatchResult {
        pub row_idx: usize,
        pub occurrence: usize,
    }
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

    // a table of rows
    #[derive(Debug)]
    pub struct Table {
        rows: Vec<Row>,
        nb_cols: usize,
        name: String,
    }

    // immutable view on a column
    #[derive(Debug)]
    pub struct Col<'a> {
        table: &'a Table,
        index: usize,
    }


    // -------------------------------------------------------------------------
    // Impl Cell
    // -------------------------------------------------------------------------

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

    // -------------------------------------------------------------------------
    // Impl Row
    // -------------------------------------------------------------------------

    impl Row {
        /// number of cells in the row
        pub fn len(&self) -> usize {
            self.cells.len()
        }

        pub fn is_empty(&self) -> bool {
            self.cells.is_empty()
        }

        /// Immutable iterator over the row cells: (column index, &Cell)
        pub fn iter_cells(&self) -> impl Iterator<Item=(usize, &Cell)> {
            self.cells.iter().enumerate()
        }

        /// Mutable iterator over the row cells: (column index, &mut Cell)
        pub fn iter_cells_mut(&mut self) -> impl Iterator<Item=(usize, &mut Cell)> {
            self.cells.iter_mut().enumerate()
        }
    }

    // -------------------------------------------------------------------------
    // Impl Col (column view)
    // -------------------------------------------------------------------------

    impl<'a> Col<'a> {
        pub fn index(&self) -> usize {
            self.index
        }

        /// Immutable iterator over the cells of this column
        pub fn iter_cells(&'a self) -> impl Iterator<Item=&'a Cell> + 'a {
            self.table
                .rows
                .iter()
                .map(move |row| &row.cells[self.index])
        }
    }

    // -------------------------------------------------------------------------
    // Impl Table
    // -------------------------------------------------------------------------

    impl Table {
        /// Create a new rectangular table `nb_rows` x `nb_cols`
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

        /// number of rows
        pub fn get_nb_rows(&self) -> usize {
            self.rows.len()
        }

        /// number of columns
        pub fn get_nb_cols(&self) -> usize {
            self.nb_cols
        }

        /// table name (sheet name)
        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn set_name<S: Into<String>>(&mut self, name: S) {
            self.name = name.into();
        }

        /// Read access to a cell
        pub fn get(&self, row: usize, col: usize) -> Option<&Cell> {
            if row < self.get_nb_rows() && col < self.nb_cols {
                self.rows.get(row)?.cells.get(col)
            } else {
                None
            }
        }

        /// Get the value of a cell as a String.
        pub fn get_cell_text(&self, row: usize, col: usize) -> String {
            self.get(row, col)
                .and_then(|c| c.value().clone())
                .unwrap_or_default()
        }

        /// Mutable access to a cell
        pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut Cell> {
            if row < self.get_nb_rows() && col < self.nb_cols {
                self.rows.get_mut(row)?.cells.get_mut(col)
            } else {
                None
            }
        }

        /// Read access to a row
        pub fn get_row(&self, row_idx: usize) -> Option<&Row> {
            self.rows.get(row_idx)
        }

        /// Mutable access to a row
        pub fn get_row_mut(&mut self, row_idx: usize) -> Option<&mut Row> {
            self.rows.get_mut(row_idx)
        }

        /// Immutable iterator over rows
        pub fn iter_rows(&self) -> impl Iterator<Item=&Row> {
            self.rows.iter()
        }

        /// Mutable iterator over rows
        pub fn iter_rows_mut(&mut self) -> impl Iterator<Item=&mut Row> {
            self.rows.iter_mut()
        }

        /// Immutable iterator over all cells: (row index, column index, &Cell)
        pub fn iter_cells(&self) -> impl Iterator<Item=(usize, usize, &Cell)> {
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

        /// Mutable iterator over all cells: (row index, column index, &mut Cell)
        pub fn iter_cells_mut(&mut self) -> impl Iterator<Item=(usize, usize, &mut Cell)> {
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

        /// Immutable iterator over columns (as views)
        pub fn iter_cols(&self) -> impl Iterator<Item=Col<'_>> {
            (0..self.nb_cols).map(move |idx| Col {
                table: self,
                index: idx,
            })
        }

        /// Get a column view (immutable)
        pub fn get_col(&self, col_idx: usize) -> Option<Col<'_>> {
            if col_idx < self.nb_cols {
                Some(Col {
                    table: self,
                    index: col_idx,
                })
            } else {
                None
            }
        }

        /// Immutable iterator over the cells of a column: (row index, &Cell)
        pub fn col_cells(
            &self,
            col_idx: usize,
        ) -> Option<impl Iterator<Item=(usize, &Cell)>> {
            if col_idx >= self.nb_cols {
                return None;
            }

            Some(
                self.rows
                    .iter()
                    .enumerate()
                    .map(move |(row_idx, row)| (row_idx, &row.cells[col_idx])),
            )
        }

        /// Mutable iterator over the cells of a column: (row index, &mut Cell)
        pub fn col_cells_mut(
            &mut self,
            col_idx: usize,
        ) -> Option<impl Iterator<Item=(usize, &mut Cell)>> {
            if col_idx >= self.nb_cols {
                return None;
            }

            Some(
                self.rows
                    .iter_mut()
                    .enumerate()
                    .map(move |(row_idx, row)| (row_idx, &mut row.cells[col_idx])),
            )
        }

        // ---------------------------------------------------------------------
        // Excel: reading
        // ---------------------------------------------------------------------

        /// Build a Table from an Excel file (first sheet or given sheet name)
        pub fn from_excel(path: &str, sheet_name: Option<&str>) -> Result<Self, Box<dyn Error>> {
            let mut workbook = open_workbook_auto(path)?;

            let sheet_name = match sheet_name {
                Some(name) => name.to_string(),
                None => workbook
                    .sheet_names()
                    .get(0)
                    .ok_or("The workbook has no sheet")?
                    .clone(),
            };

            let range = match workbook.worksheet_range(&sheet_name) {
                Ok(r) => r,
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

            // Pad rows so the table is rectangular
            for row in &mut rows {
                while row.cells.len() < max_cols {
                    row.cells.push(Cell { value: None });
                }
            }

            Ok(Table {
                rows,
                nb_cols: max_cols,
                name: sheet_name,
            })
        }

        // ---------------------------------------------------------------------
        // Excel: writing
        // ---------------------------------------------------------------------

        /// Write multiple tables into an Excel file, 1 sheet per table
        pub fn write_tables_to_excel(
            path: &str,
            sheets: Vec<Table>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let path = std::path::Path::new(path);
            let mut book = new_file();
            for (i, sheet) in sheets.into_iter().enumerate() {
                book.new_sheet(sheet.name.clone())?;
                for (r, row) in sheet.iter_rows().enumerate() {
                    for (c, cell) in row.cells.iter().enumerate() {
                        if let Some(value) = &cell.value {
                            book.get_sheet_mut(&(i + 1)).unwrap().get_cell_mut((c as u32 + 1, r as u32 + 1)).set_value_string(value);
                        }
                    }
                }
            }
            book.remove_sheet(0)?;
            writer::xlsx::write(&book, path)?;
            Ok(())
        }

        /// Write a single table into an Excel file
        pub fn to_excel(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
            let path = std::path::Path::new(path);
            let mut book = new_file();
            book.new_sheet(self.name.clone())?;
            for (r, row) in self.iter_rows().enumerate() {
                for (c, cell) in row.cells.iter().enumerate() {
                    if let Some(value) = &cell.value {
                        book.get_sheet_mut(&(1)).unwrap().get_cell_mut((c as u32 + 1, r as u32 + 1)).set_value_string(value);
                    }
                }
            }
            book.remove_sheet(0)?;
            writer::xlsx::write(&book, path)?;
            Ok(())
        }

        /// Insert an empty row at the given index (0..=rows()).
        /// Returns Err if index is out of range.
        pub fn insert_row(&mut self, row_idx: usize) -> Result<(), String> {
            if row_idx > self.get_nb_rows() {
                return Err(format!(
                    "row index {} out of range (0..={})",
                    row_idx,
                    self.get_nb_rows()
                ));
            }

            let mut cells = Vec::with_capacity(self.nb_cols);
            for _ in 0..self.nb_cols {
                cells.push(Cell { value: None });
            }

            self.rows.insert(row_idx, Row { cells });
            Ok(())
        }

        /// Remove the row at the given index and return it.
        pub fn remove_row(&mut self, row_idx: usize) -> Result<Row, String> {
            if row_idx >= self.get_nb_rows() {
                return Err(format!(
                    "row index {} out of range (0..{})",
                    row_idx,
                    self.get_nb_rows().saturating_sub(1)
                ));
            }

            Ok(self.rows.remove(row_idx))
        }

        /// Add one empty row at the end (alias around insert_row).
        pub fn add_row(&mut self) {
            // ignore the error because rows() is always a valid insertion index
            let _ = self.insert_row(self.get_nb_rows());
        }

        /// Add `n` empty rows at the end.
        pub fn add_rows(&mut self, n: usize) {
            for _ in 0..n {
                self.add_row();
            }
        }

        /// Insert an empty column at the given index (0..=cols()).
        /// All rows grow by one cell. If there are no rows, only nb_cols is updated.
        pub fn insert_col(&mut self, col_idx: usize) -> Result<(), String> {
            if col_idx > self.nb_cols {
                return Err(format!(
                    "column index {} out of range (0..={})",
                    col_idx,
                    self.nb_cols
                ));
            }

            for row in &mut self.rows {
                row.cells.insert(col_idx, Cell { value: None });
            }

            self.nb_cols += 1;
            Ok(())
        }

        /// Add an empty column at the end.
        pub fn add_col(&mut self) -> Result<usize, String> {
            self.insert_col(self.nb_cols)?;
            Ok(self.nb_cols - 1)
        }

        /// Remove the column at the given index.
        /// All rows shrink by one cell.
        pub fn remove_col(&mut self, col_idx: usize) -> Result<(), String> {
            if col_idx >= self.nb_cols {
                return Err(format!(
                    "column index {} out of range (0..{})",
                    col_idx,
                    self.nb_cols.saturating_sub(1)
                ));
            }

            for row in &mut self.rows {
                if !row.cells.is_empty() {
                    row.cells.remove(col_idx);
                }
            }

            self.nb_cols -= 1;
            Ok(())
        }

        /// Convert an Excel column label (e.g. "A", "Z", "AA") to a 1-based index.
        /// Returns None if the string is empty or contains non-letters.
        pub fn str_col_to_index(col: &str) -> Option<usize> {
            if col.is_empty() {
                return None;
            }

            let mut value: usize = 0;

            for ch in col.chars() {
                let upper = ch.to_ascii_uppercase();
                if !upper.is_ascii_alphabetic() {
                    return None;
                }

                let digit = (upper as u8 - b'A' + 1) as usize; // 'A' -> 1, 'B' -> 2, ...
                value = value * 26 + digit;
            }

            Some(value)
        }

        // return each occurrence of the searched value and the number of times it was found
        pub fn find_occurrences_in_column(
            &self,
            col_idx: usize,
            needle: &str,
            mode: MatchMode,
        ) -> Result<Vec<MatchResult>, String> {
            if col_idx >= self.get_nb_cols() {
                return Err(format!(
                    "Indice de colonne invalide: {} (max = {})",
                    col_idx,
                    self.get_nb_cols().saturating_sub(1)
                ));
            }

            let mut result = Vec::new();
            let mut occurrence = 0usize;

            for row_idx in 0..self.get_nb_rows() {
                let text = self.get_cell_text(row_idx, col_idx);

                let found = match mode {
                    MatchMode::Exact => text == needle,
                    MatchMode::Contains => text.contains(needle),
                };

                if found {
                    occurrence += 1;
                    result.push(MatchResult{row_idx, occurrence});
                }
            }

            Ok(result)
        }

        pub fn find_duplicates_in_column(&self, col_idx: usize) -> Result<Vec<MatchResult>, String> {
            if col_idx >= self.get_nb_cols() {
                return Err(format!(
                    "Indice de colonne invalide: {} (max = {})",
                    col_idx,
                    self.get_nb_cols().saturating_sub(1)
                ));
            }
            let mut counts: HashMap<String, usize> = HashMap::new();
            let mut result: Vec<MatchResult> = Vec::new();

            for row_idx in 0..self.get_nb_rows() {
                let value = self.get_cell_text(row_idx, col_idx);
                let count = counts.entry(value).or_insert(0);
                *count += 1;
                if *count >= 2 {
                    result.push(MatchResult{row_idx, occurrence:*count});
                } else {
                    result.push(MatchResult{row_idx, occurrence:*count});
                }
            }
            Ok(result)
        }
    }
}
