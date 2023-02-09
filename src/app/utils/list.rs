use tui::widgets::ListState;

// https://github.com/fdehau/tui-rs/blob/master/examples/list.rs
pub struct List<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T: Clone + Eq> List<T> {
    pub fn with_items(items: Vec<T>) -> List<T> {
        let mut list = List { state: ListState::default(), items };

        if !list.items.is_empty() {
            list.state.select(Some(0));
        }

        list
    }

    pub fn next(&mut self) {
        if !self.items.is_empty() {
            let i = match self.state.selected() {
                Some(i) => {
                    if i >= self.items.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        if !self.items.is_empty() {
            let i = match self.state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    pub fn select(&mut self, idx: usize) {
        if idx < self.items.len() {
            self.state.select(Some(idx));
        }
    }

    pub fn selected(&self) -> Option<T> {
        self.state.selected().map(|i| self.items[i].clone())
    }

    pub fn find(&self, key: T) -> Option<usize> {
        self.items.iter().position(|item| *item == key)
    }
}
