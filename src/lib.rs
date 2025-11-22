pub mod table {
    use calamine::{Data, Range, Reader, open_workbook_auto};
    use std::collections::HashMap;
    use std::error::Error;
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
        pub header: Option<Header>,
    }

    // immutable view on a column
    #[derive(Debug)]
    pub struct Col<'a> {
        table: &'a Table,
        index: usize,
    }

    #[derive(Clone, Debug)]
    pub struct Header {
        titles: Vec<String>,
    }

    impl Header {
        pub fn new(titles: Vec<String>) -> Self {
            Header { titles }
        }

        pub fn len(&self) -> usize {
            self.titles.len()
        }

        pub fn is_empty(&self) -> bool {
            self.titles.is_empty()
        }

        /// Get the title at the given index.
        pub fn get_title(&self, idx: usize) -> Option<&str> {
            self.titles.get(idx).map(|s| s.as_str())
        }

        /// Immutable iterator over the titles: &str
        pub fn iter(&self) -> impl Iterator<Item = &str> {
            self.titles.iter().map(|s| s.as_str())
        }

        // Set the title at the given index (without change the number of columns)
        pub fn set_title<S: Into<String>>(&mut self, idx: usize, title: S) -> Result<(), String> {
            if idx >= self.titles.len() {
                return Err(format!(
                    "header column index {} out of range (0..={})",
                    idx,
                    self.titles.len().saturating_sub(1)
                ));
            }
            self.titles[idx] = title.into();
            Ok(())
        }

        pub fn ensure_len(&mut self, len: usize) {
            if self.titles.len() < len {
                self.titles.resize(len, String::new());
            }
        }

        pub fn find_col_index_by_title(&self, title: &str) -> Option<usize> {
            self.titles.iter().position(|t| t == title)
        }

        /// Return a list of column indices corresponding to the given titles.
        /// Returns an error if any title is not found.
        pub fn find_cols_indices_by_titles(&self, titles: &[&str]) -> Result<Vec<usize>, String> {
            let mut indices = Vec::with_capacity(titles.len());

            for &title in titles {
                match self.find_col_index_by_title(title) {
                    Some(idx) => indices.push(idx),
                    None => {
                        return Err(format!("Column title '{}' not found in header", title));
                    }
                }
            }

            Ok(indices)
        }
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
        
        pub fn get_value_as_str (&self) -> String {
            self.value.clone().unwrap_or_default()
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
        pub fn iter_cells(&self) -> impl Iterator<Item = (usize, &Cell)> {
            self.cells.iter().enumerate()
        }

        /// Mutable iterator over the row cells: (column index, &mut Cell)
        pub fn iter_cells_mut(&mut self) -> impl Iterator<Item = (usize, &mut Cell)> {
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
        pub fn iter_cells(&'a self) -> impl Iterator<Item = &'a Cell> + 'a {
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
            Self {
                rows,
                nb_cols,
                name,
                header: None,
            }
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
        pub fn iter_rows(&self) -> impl Iterator<Item = &Row> {
            self.rows.iter()
        }

        /// Mutable iterator over rows
        pub fn iter_rows_mut(&mut self) -> impl Iterator<Item = &mut Row> {
            self.rows.iter_mut()
        }

        /// Immutable iterator over all cells: (row index, column index, &Cell)
        pub fn iter_cells(&self) -> impl Iterator<Item = (usize, usize, &Cell)> {
            self.rows.iter().enumerate().flat_map(|(row_idx, row)| {
                row.cells
                    .iter()
                    .enumerate()
                    .map(move |(col_idx, cell)| (row_idx, col_idx, cell))
            })
        }

        /// Mutable iterator over all cells: (row index, column index, &mut Cell)
        pub fn iter_cells_mut(&mut self) -> impl Iterator<Item = (usize, usize, &mut Cell)> {
            self.rows.iter_mut().enumerate().flat_map(|(row_idx, row)| {
                row.cells
                    .iter_mut()
                    .enumerate()
                    .map(move |(col_idx, cell)| (row_idx, col_idx, cell))
            })
        }

        /// Immutable iterator over columns (as views)
        pub fn iter_cols(&self) -> impl Iterator<Item = Col<'_>> {
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
        pub fn col_cells(&self, col_idx: usize) -> Option<impl Iterator<Item = (usize, &Cell)>> {
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
        ) -> Option<impl Iterator<Item = (usize, &mut Cell)>> {
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
        fn from_range(range: &Range<Data>, sheet_name: String) -> Self {
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

            // rendre le tableau rectangulaire
            for row in &mut rows {
                while row.cells.len() < max_cols {
                    row.cells.push(Cell { value: None });
                }
            }

            Table {
                rows,
                nb_cols: max_cols,
                name: sheet_name,
                header: None,
            }
        }

        /// Version actuelle : charge une seule feuille (par nom ou la première)
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

            Ok(Table::from_range(&range, sheet_name))
        }

        /// Nouvelle fonction : charge **toutes** les feuilles et retourne un Vec<Table>
        pub fn from_excel_all_sheets(path: &str) -> Result<Vec<Self>, Box<dyn Error>> {
            let mut workbook = open_workbook_auto(path)?;
            let sheet_names = workbook.sheet_names().clone(); // on clone la liste des noms

            let mut tables = Vec::new();

            for sheet_name in sheet_names {
                let range = match workbook.worksheet_range(&sheet_name) {
                    Ok(r) => r,
                    Err(e) => return Err(Box::new(e)),
                };

                let table = Table::from_range(&range, sheet_name);
                tables.push(table);
            }

            Ok(tables)
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
                            book.get_sheet_mut(&(i + 1))
                                .unwrap()
                                .get_cell_mut((c as u32 + 1, r as u32 + 1))
                                .set_value_string(value);
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
                        book.get_sheet_mut(&(1))
                            .unwrap()
                            .get_cell_mut((c as u32 + 1, r as u32 + 1))
                            .set_value_string(value);
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
        pub fn add_row(&mut self) -> usize {
            // ignore the error because rows() is always a valid insertion index
            let _ =self.insert_row(self.get_nb_rows());
            self.get_nb_rows()-1
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
                    col_idx, self.nb_cols
                ));
            }

            for row in &mut self.rows {
                row.cells.insert(col_idx, Cell { value: None });
            }

            if let Some(ref mut header) = self.header {
                // numéro humain de la colonne (1-based)
                let col_number = col_idx + 1;
                let title = format!("Head_Col_{}", col_number);

                // normalement header.titles.len() == nb_cols,
                // mais on se protège au cas où.
                if col_idx <= header.titles.len() {
                    header.titles.insert(col_idx, title);
                } else {
                    // cas de désynchro : on pad jusqu'à col_idx
                    while header.titles.len() < col_idx {
                        header.titles.push(String::new());
                    }
                    header.titles.push(title);
                }
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

            if let Some(ref mut header) = self.header {
                if col_idx < header.titles.len() {
                    header.titles.remove(col_idx);
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
                    result.push(MatchResult {
                        row_idx,
                        occurrence,
                    });
                }
            }

            Ok(result)
        }

        // return each row where the searched value is found
        pub fn find_duplicates_in_column(
            &self,
            col_idx: usize,
        ) -> Result<Vec<MatchResult>, String> {
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
                    result.push(MatchResult {
                        row_idx,
                        occurrence: *count,
                    });
                } else {
                    result.push(MatchResult {
                        row_idx,
                        occurrence: *count,
                    });
                }
            }
            Ok(result)
        }

        // remove the header from the data and create a real header
        pub fn first_line_as_header(&mut self) -> Result<&Header, String> {
            if self.header.is_some() {
                return Err("Le header a déjà été extrait".into());
            }

            if self.rows.is_empty() {
                return Err("Impossible d'extraire un header d'un tableau vide".into());
            }

            // On enlève la première ligne
            let header_row = self.rows.remove(0);

            // On convertit les cellules en titres (None => "")
            let mut titles = Vec::with_capacity(self.nb_cols);
            for cell in header_row.cells.into_iter().take(self.nb_cols) {
                titles.push(cell.value.unwrap_or_default());
            }

            // S'il manque des colonnes, on complète avec ""
            while titles.len() < self.nb_cols {
                titles.push(String::new());
            }

            self.header = Some(Header { titles });

            // On renvoie une référence sur le header pour usage immédiat
            Ok(self.header.as_ref().unwrap())
        }

        // insert as first row with the header titles but keep the original header
        pub fn insert_header_as_first_row(&mut self) -> Result<(), String> {
            let header = match &self.header {
                Some(h) => h,
                None => return Err("Aucun header défini, impossible de le réinsérer".into()),
            };

            // Construire la ligne à partir des titres du header
            let mut cells = Vec::with_capacity(self.nb_cols);

            // On met autant de colonnes que `nb_cols`
            for col_idx in 0..self.nb_cols {
                let title = header.titles.get(col_idx).cloned().unwrap_or_default(); // si pas de titre, chaîne vide

                cells.push(Cell { value: Some(title) });
            }

            // On insère cette ligne en première position
            self.rows.insert(0, Row { cells });

            Ok(())
        }

        // move the header to the first row and remove the original header
        pub fn move_header_to_first_row(&mut self) -> Result<(), String> {
            let header = self
                .header
                .take()
                .ok_or_else(|| "Aucun header défini, impossible de le réinsérer".to_string())?;

            let mut cells = Vec::with_capacity(self.nb_cols);

            for col_idx in 0..self.nb_cols {
                let title = header.titles.get(col_idx).cloned().unwrap_or_default();

                cells.push(Cell { value: Some(title) });
            }

            self.rows.insert(0, Row { cells });

            Ok(())
        }

        // return the header
        pub fn get_header(&self) -> Result<&Header, String> {
            self.header
                .as_ref()
                .ok_or_else(|| "No header defined".to_string())
        }

        // set the title of a column in the header
        pub fn set_header_title<S: Into<String>>(
            &mut self,
            col_idx: usize,
            title: S,
        ) -> Result<(), String> {
            // Vérifier que la colonne existe dans le tableau
            if col_idx >= self.nb_cols {
                return Err(format!(
                    "column index {} out of range (0..={})",
                    col_idx,
                    self.nb_cols.saturating_sub(1)
                ));
            }

            let header = self
                .header
                .as_mut()
                .ok_or_else(|| "No header defined".to_string())?;

            // Assurer que le header a au moins nb_cols entrées
            header.ensure_len(self.nb_cols);

            // Maintenant on peut utiliser la méthode du Header
            header.set_title(col_idx, title)
        }

        // return the index of the column with the given title
        pub fn get_col_index_by_title(&self, title: &str) -> Result<usize, String> {
            let header = self
                .header
                .as_ref()
                .ok_or_else(|| "No header defined".to_string())?;

            header
                .find_col_index_by_title(title)
                .ok_or_else(|| format!("Column title '{}' not found in header", title))
        }

        // remove the columns with the given indices
        pub fn remove_cols(&mut self, col_indices: &[usize]) -> Result<(), String> {
            if col_indices.is_empty() {
                return Ok(());
            }

            // copy the indices into a Vec to sort them
            let mut idxs: Vec<usize> = col_indices.to_vec();

            // sort and remove duplicates
            idxs.sort_unstable();
            idxs.dedup();

            // we remove the columns in reverse order
            for &idx in idxs.iter().rev() {
                self.remove_col(idx)?;
            }

            Ok(())
        }

        /// Keeps only the columns whose indices are given in `col_indices`.
        /// All other columns are removed.
        ///
        /// - `col_indices` can be in any order.
        /// - Duplicates are ignored.
        /// - Returns an error if any index is out of bounds.
        pub fn keep_cols(&mut self, col_indices: &[usize]) -> Result<(), String> {
            // If we don't want to keep any column: remove all existing columns
            if col_indices.is_empty() {
                let to_remove: Vec<usize> = (0..self.nb_cols).collect();
                return self.remove_cols(&to_remove);
            }

            // Copy / normalize the indices to keep
            let mut keep: Vec<usize> = col_indices.to_vec();
            keep.sort_unstable();
            keep.dedup();

            // Validate indices (since they are sorted, we only need to check the largest)
            if let Some(&max_idx) = keep.last() {
                if max_idx >= self.nb_cols {
                    return Err(format!(
                        "column index {} out of range (0..={})",
                        max_idx,
                        self.nb_cols.saturating_sub(1)
                    ));
                }
            }

            // Build the list of columns to remove = complement of `keep`
            let mut to_remove = Vec::new();
            let mut it_keep = keep.iter().copied();
            let mut current_keep = it_keep.next();

            for idx in 0..self.nb_cols {
                if Some(idx) == current_keep {
                    // This column is in the "keep" list, so we skip it
                    current_keep = it_keep.next();
                } else {
                    // This column is not in the "keep" list, so we will remove it
                    to_remove.push(idx);
                }
            }

            // Reuse existing logic that already updates rows + header correctly
            self.remove_cols(&to_remove)
        }

        /// Return the indices of the columns whose titles are given in `titles`.
        /// Fails if no header is defined or if any title is not found.
        pub fn get_cols_indices_by_titles(&self, titles: &[&str]) -> Result<Vec<usize>, String> {
            let header = self
                .header
                .as_ref()
                .ok_or_else(|| "No header defined".to_string())?;

            header.find_cols_indices_by_titles(titles)
        }

        /// Create a new empty table from a list of column titles.
        pub fn from_column_titles<S, N>(titles: &[S], name: N) -> Self
        where
            S: AsRef<str>,
            N: Into<String>,
        {
            // Convert the slice of titles into owned Strings
            let titles_vec: Vec<String> = titles
                .iter()
                .map(|t| t.as_ref().to_string())
                .collect();

            let header = Header::new(titles_vec);
            let nb_cols = header.len();

            Table {
                header: Some(header),
                rows: Vec::new(),     // no data rows yet
                nb_cols,
                name: name.into(),
            }
        }

        pub fn set_cell_value(
            &mut self,
            row_idx: usize,
            col_idx: usize,
            value: Value,
        ) -> Result<(), String> {
            // check row index
            if row_idx >= self.rows.len() {
                return Err(format!(
                    "row index {} out of range (0..={})",
                    row_idx,
                    self.rows.len().saturating_sub(1)
                ));
            }

            // check column index
            if col_idx >= self.nb_cols {
                return Err(format!(
                    "column index {} out of range (0..={})",
                    col_idx,
                    self.nb_cols.saturating_sub(1)
                ));
            }

            // set the value
            self.rows[row_idx].cells[col_idx].value = value;
            Ok(())
        }

        /// Return the list of row indices where the value in column `col_idx`
        /// matches `needle` exactly (using Value instead of &str).
        ///
        /// - Returns an error if `col_idx` is out of range.
        pub fn find_rows_matching_value(
            &self,
            col_idx: usize,
            needle: &Value,
        ) -> Result<Vec<usize>, String> {
            // Check column index
            if col_idx >= self.nb_cols {
                return Err(format!(
                    "column index {} out of range (0..={})",
                    col_idx,
                    self.nb_cols.saturating_sub(1)
                ));
            }

            let mut matches = Vec::new();

            for (row_idx, row) in self.rows.iter().enumerate() {
                // Directly compare the cell's Value with the needle
                if let Some(cell) = row.cells.get(col_idx) {
                    if &cell.value == needle {
                        matches.push(row_idx);
                    }
                }
            }

            Ok(matches)
        }
        /// Set the value of a cell by row index and column title.
        ///
        /// Errors:
        /// - if there is no header
        /// - if the column title is not found in the header
        /// - if the row index is out of range
        pub fn set_cell_value_by_title(
            &mut self,
            row_idx: usize,
            col_title: &str,
            value: Value,
        ) -> Result<(), String> {
            // 1) Find the column index from the header
            let col_idx = self.get_col_index_by_title(col_title)?;

            // 2) Delegate to the low-level setter using indices
            self.set_cell_value(row_idx, col_idx, value)
        }
    }
}
