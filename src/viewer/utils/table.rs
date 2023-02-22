use rayon::prelude::*;
use tui::widgets::TableState;

pub(crate) struct Table {
    pub state: TableState,
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new(header: &[&str], rows: &[&[&str]]) -> Table {
        let mut state = TableState::default();

        if !rows.is_empty() {
            state.select(Some(0));
        }

        let header: Vec<String> = header.par_iter().map(|s| s.to_string()).collect();

        let rows: Vec<Vec<String>> =
            rows.par_iter().map(|row| row.iter().map(|s| s.to_string()).collect()).collect();

        Table { state, header, rows }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.rows.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.rows.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
