//! hcscoder Notebook Edit Tool
//!
//! Jupyter notebook reading and editing.
//! Zero telemetry, no phone-home logic.

use crate::hcscoder_tools::filesystem;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Notebook cell type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CellType {
    Code,
    Markdown,
    Raw,
}

/// Notebook cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookCell {
    pub cell_type: CellType,
    pub source: Vec<String>,
    pub outputs: Option<Vec<serde_json::Value>>,
    pub execution_count: Option<u32>,
    pub metadata: serde_json::Value,
}

/// Jupyter notebook structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub cells: Vec<NotebookCell>,
    pub metadata: serde_json::Value,
    pub nbformat: u32,
    pub nbformat_minor: u32,
}

impl Notebook {
    /// Read notebook from file
    pub async fn read(path: &str) -> Result<Self> {
        let content = filesystem::read_file(path).await?;
        serde_json::from_str(&content).context("Failed to parse notebook JSON")
    }

    /// Write notebook to file
    pub async fn write(&self, path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        filesystem::write_file(path, &content).await
    }

    /// Get cell count
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Insert a cell at position
    pub fn insert_cell(&mut self, index: usize, cell: NotebookCell) {
        if index > self.cells.len() {
            self.cells.push(cell);
        } else {
            self.cells.insert(index, cell);
        }
    }

    /// Delete a cell
    pub fn delete_cell(&mut self, index: usize) -> Option<NotebookCell> {
        if index < self.cells.len() {
            Some(self.cells.remove(index))
        } else {
            None
        }
    }

    /// Update cell source
    pub fn update_cell_source(&mut self, index: usize, source: Vec<String>) -> Result<()> {
        let cell = self
            .cells
            .get_mut(index)
            .context(format!("Cell index {} out of bounds", index))?;
        cell.source = source;
        Ok(())
    }
}

/// Read a notebook file
pub async fn read_notebook(path: &str) -> Result<Notebook> {
    Notebook::read(path).await
}

/// Write a notebook file
pub async fn write_notebook(notebook: &Notebook, path: &str) -> Result<()> {
    notebook.write(path).await
}

/// Execute a cell (experimental; requires kernel integration)
pub async fn execute_cell(path: &str, cell_index: usize) -> Result<String> {
    Err(anyhow::anyhow!(
        "Notebook cell execution is experimental and requires kernel integration. Path: {}, Cell: {}",
        path,
        cell_index
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notebook_operations() {
        let mut notebook = Notebook {
            cells: vec![],
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            nbformat: 4,
            nbformat_minor: 5,
        };

        let cell = NotebookCell {
            cell_type: CellType::Code,
            source: vec!["print('Hello')".to_string()],
            outputs: None,
            execution_count: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        };

        notebook.insert_cell(0, cell.clone());
        assert_eq!(notebook.cell_count(), 1);
    }
}
