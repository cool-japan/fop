//! Table-related FO elements
//!
//! Implements fo:table, fo:table-column, fo:table-body, fo:table-row, fo:table-cell

use crate::properties::PropertyList;

/// Table element (fo:table)
#[derive(Debug, Clone)]
pub struct Table<'a> {
    /// Property list
    pub properties: PropertyList<'a>,

    /// Table columns (fo:table-column)
    pub columns: Vec<TableColumn<'a>>,

    /// Table header (optional)
    pub header: Option<TableBody<'a>>,

    /// Table body
    pub body: Vec<TableBody<'a>>,

    /// Table footer (optional)
    pub footer: Option<TableBody<'a>>,

    /// Border collapse model (separate or collapse)
    pub border_collapse: BorderCollapse,
}

/// Table column definition (fo:table-column)
#[derive(Debug, Clone)]
pub struct TableColumn<'a> {
    /// Property list
    pub properties: PropertyList<'a>,

    /// Column width (explicit or proportional)
    pub column_width: ColumnWidth,

    /// Number of times to repeat this column
    pub number_columns_repeated: usize,
}

/// Table body, header, or footer (fo:table-body, fo:table-header, fo:table-footer)
#[derive(Debug, Clone)]
pub struct TableBody<'a> {
    /// Property list
    pub properties: PropertyList<'a>,

    /// Table rows
    pub rows: Vec<TableRow<'a>>,
}

/// Table row (fo:table-row)
#[derive(Debug, Clone)]
pub struct TableRow<'a> {
    /// Property list
    pub properties: PropertyList<'a>,

    /// Table cells
    pub cells: Vec<TableCell<'a>>,

    /// Row height (optional)
    pub height: Option<crate::Length>,
}

/// Table cell (fo:table-cell)
#[derive(Debug, Clone)]
pub struct TableCell<'a> {
    /// Property list
    pub properties: PropertyList<'a>,

    /// Number of columns spanned
    pub number_columns_spanned: usize,

    /// Number of rows spanned
    pub number_rows_spanned: usize,

    /// Cell content (blocks)
    pub blocks: Vec<super::Block<'a>>,
}

/// Border collapse model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderCollapse {
    /// Separate borders (default)
    Separate,

    /// Collapsed borders
    Collapse,
}

impl Default for BorderCollapse {
    fn default() -> Self {
        Self::Separate
    }
}

/// Column width specification
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnWidth {
    /// Fixed width
    Fixed(crate::Length),

    /// Proportional width (e.g., "2*" means 2 units)
    Proportional(f64),

    /// Auto width (calculated from content)
    Auto,
}

impl<'a> Table<'a> {
    /// Create a new table
    pub fn new(properties: PropertyList<'a>) -> Self {
        Self {
            properties,
            columns: Vec::new(),
            header: None,
            body: Vec::new(),
            footer: None,
            border_collapse: BorderCollapse::default(),
        }
    }

    /// Get the total number of columns
    pub fn column_count(&self) -> usize {
        self.columns.iter()
            .map(|col| col.number_columns_repeated)
            .sum()
    }

    /// Get column width for a specific column index
    pub fn column_width(&self, index: usize) -> Option<&ColumnWidth> {
        let mut current_idx = 0;
        for col in &self.columns {
            if current_idx <= index && index < current_idx + col.number_columns_repeated {
                return Some(&col.column_width);
            }
            current_idx += col.number_columns_repeated;
        }
        None
    }
}

impl<'a> TableColumn<'a> {
    /// Create a new table column
    pub fn new(properties: PropertyList<'a>, column_width: ColumnWidth) -> Self {
        Self {
            properties,
            column_width,
            number_columns_repeated: 1,
        }
    }
}

impl<'a> TableBody<'a> {
    /// Create a new table body
    pub fn new(properties: PropertyList<'a>) -> Self {
        Self {
            properties,
            rows: Vec::new(),
        }
    }

    /// Get the number of rows
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

impl<'a> TableRow<'a> {
    /// Create a new table row
    pub fn new(properties: PropertyList<'a>) -> Self {
        Self {
            properties,
            cells: Vec::new(),
            height: None,
        }
    }

    /// Get the number of cells
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }
}

impl<'a> TableCell<'a> {
    /// Create a new table cell
    pub fn new(properties: PropertyList<'a>) -> Self {
        Self {
            properties,
            number_columns_spanned: 1,
            number_rows_spanned: 1,
            blocks: Vec::new(),
        }
    }

    /// Check if cell spans multiple columns
    pub fn spans_columns(&self) -> bool {
        self.number_columns_spanned > 1
    }

    /// Check if cell spans multiple rows
    pub fn spans_rows(&self) -> bool {
        self.number_rows_spanned > 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_creation() {
        let props = PropertyList::new();
        let table = Table::new(props);

        assert_eq!(table.column_count(), 0);
        assert!(table.header.is_none());
        assert_eq!(table.body.len(), 0);
    }

    #[test]
    fn test_column_count() {
        let props = PropertyList::new();
        let mut table = Table::new(props);

        let col1 = TableColumn::new(PropertyList::new(), ColumnWidth::Auto);
        let mut col2 = TableColumn::new(PropertyList::new(), ColumnWidth::Proportional(2.0));
        col2.number_columns_repeated = 3;

        table.columns.push(col1);
        table.columns.push(col2);

        assert_eq!(table.column_count(), 4); // 1 + 3
    }

    #[test]
    fn test_column_width_lookup() {
        let props = PropertyList::new();
        let mut table = Table::new(props);

        let col1 = TableColumn::new(PropertyList::new(), ColumnWidth::Fixed(crate::Length::from_pt(100.0)));
        let mut col2 = TableColumn::new(PropertyList::new(), ColumnWidth::Auto);
        col2.number_columns_repeated = 2;

        table.columns.push(col1);
        table.columns.push(col2);

        // Column 0 should be Fixed
        match table.column_width(0) {
            Some(ColumnWidth::Fixed(len)) => assert_eq!(*len, crate::Length::from_pt(100.0)),
            _ => panic!("Expected Fixed width"),
        }

        // Columns 1 and 2 should be Auto
        assert!(matches!(table.column_width(1), Some(ColumnWidth::Auto)));
        assert!(matches!(table.column_width(2), Some(ColumnWidth::Auto)));

        // Column 3 doesn't exist
        assert!(table.column_width(3).is_none());
    }

    #[test]
    fn test_cell_spanning() {
        let props = PropertyList::new();
        let mut cell = TableCell::new(props);

        assert!(!cell.spans_columns());
        assert!(!cell.spans_rows());

        cell.number_columns_spanned = 2;
        cell.number_rows_spanned = 3;

        assert!(cell.spans_columns());
        assert!(cell.spans_rows());
    }

    #[test]
    fn test_border_collapse_default() {
        let props = PropertyList::new();
        let table = Table::new(props);

        assert_eq!(table.border_collapse, BorderCollapse::Separate);
    }
}
