use crate::viewer::{
    command::{Command, CommandTrie},
    error::{DotViewerError, DotViewerResult},
    help::Help,
    modes::{Mode, PopupMode, SearchMode},
    success::SuccessState,
    utils::{Input, List, Tabs},
    view::View,
};

use dot_graph::{parser, Graph};

/// `App` holds `dot-viewer` application states.
///
/// `tui-rs` simply redraws the entire screen in a loop while accepting keyboard inputs.
/// Thus `App` should keep track of the application context in its fields.
pub(crate) struct App {
    /// Whether to quit the application or not, by `q` keybinding
    pub quit: bool,

    /// Current mode the application is in
    pub mode: Mode,

    /// Result of the last command that was made
    pub result: DotViewerResult<SuccessState>,

    /// Tabs to be shown in the main screen
    pub tabs: Tabs<View>,

    /// Input form to be shown in the main screen
    pub input: Input,

    /// Autocomplete support for commands
    pub(crate) trie: CommandTrie,

    /// Keybinding helps
    pub help: Help,
}

impl App {
    /// Constructs a new `App`, given a `path` to a dot format DAG.
    pub(crate) fn new(path: &str) -> DotViewerResult<App> {
        let quit = false;

        let mode = Mode::Normal;

        let result: DotViewerResult<SuccessState> = Ok(SuccessState::default());

        let graph = parser::parse(path)?;
        let view = View::new(graph.id().clone(), graph);
        let tabs = Tabs::from_iter(vec![view]);

        let input = Input::default();

        let trie = CommandTrie::new();

        let help = Help::new();

        Ok(App { quit, mode, result, tabs, input, trie, help })
    } 

    /// Parse and execute dot-viewer command
    pub(crate) fn exec(&mut self) -> DotViewerResult<SuccessState> {
        let command = Command::parse(&self.input.key);
        self.set_normal_mode();

        match command {
            Command::Filter(filter) => filter.prefix.map_or(
                Err(DotViewerError::CommandError("No argument supplied for filter".to_string())),
                |prefix| self.filter(&prefix).map(|_| SuccessState::default())
            ),
            Command::Neighbors(neighbors) => neighbors.depth.map_or(
                Err(DotViewerError::CommandError("No argument supplied for neighbors".to_string())),
                |depth| self.neighbors(depth)
            ),
            Command::Help => {
                self.set_popup_mode(PopupMode::Help);
                Ok(SuccessState::default())
            }
            Command::Subgraph => {
                self.set_popup_mode(PopupMode::Tree);
                Ok(SuccessState::default())
            },
            Command::Export => self.export(),
            Command::Xdot => self.xdot(),
            Command::NoMatch => {
                let key = &self.input.key;
                Err(DotViewerError::CommandError(format!("No such command {key}")))
            }
        }
    }

    /// Autocomplete user input
    pub(crate) fn autocomplete_fuzzy(&mut self) {
        let view = self.tabs.selected();

        let key = &self.input.key;
        if let Some(key) = view.autocomplete(key) {
            view.update_fuzzy(&key);
            view.update_trie();
            self.input.set(key);
        }
    }

    /// Autocomplete user input
    pub(crate) fn autocomplete_regex(&mut self) {
        let view = self.tabs.selected();

        let key = &self.input.key;
        if let Some(key) = view.autocomplete(key) {
            view.update_regex(&key);
            view.update_trie();
            self.input.set(key);
        }
    }

    /// Autocomplete user input 
    pub(crate) fn autocomplete_command(&mut self) {
        let command = Command::parse(&self.input.key);

        match command {
            Command::NoMatch => self.autocomplete_cmd(),
            _ => {},
        }
    }

    fn autocomplete_cmd(&mut self) {
        let cmd = &self.input.key;
        if let Some(cmd) = self.trie.trie_cmd.autocomplete(cmd) {
            self.input.set(cmd);
        }
    }

    /// Apply prefix filter on the current view.
    /// Based on the currently typed input, it applies a prefix filter on the current view,
    /// and opens a new tab with the filtered view.
    pub(crate) fn filter(&mut self, prefix: &str) -> DotViewerResult<()> {
        let view_current = self.tabs.selected();
        let view_new = view_current.filter(prefix)?;
        self.tabs.open(view_new);

        Ok(())
    }

    /// Extract a subgraph from the current view.
    /// When a subgraph id is selected in the subgraph tree,
    /// it opens a new tab containing only the selected subgraph.
    pub(crate) fn subgraph(&mut self) -> DotViewerResult<()> {
        let view_current = self.tabs.selected();
        let view_new = view_current.subgraph()?;
        self.tabs.open(view_new);

        self.set_normal_mode();

        Ok(())
    }

    /// Export a neigbor graph from the currently selected node to dot,
    /// given the neighbor depth by `0-9` keybindings.
    pub(crate) fn neighbors(&mut self, depth: usize) -> DotViewerResult<SuccessState> {
        let view = self.tabs.selected();
        let graph = &view.graph;
        let node = &view.current_id();

        let filename = format!("{node}-{depth}");

        let neighbor_graph =
            graph.neighbors(node, depth).map_err(|e| DotViewerError::ViewerError(e.to_string()))?;

        if neighbor_graph.is_empty() {
            return Err(DotViewerError::ViewerError("empty graph".to_string()));
        }

        write_graph(filename, &neighbor_graph)
    }

    /// Export the current view to dot.
    pub(crate) fn export(&mut self) -> DotViewerResult<SuccessState> {
        let viewer = self.tabs.selected();
        let graph = &viewer.graph;

        let filename: String = viewer.title.chars().filter(|c| !c.is_whitespace()).collect();

        write_graph(filename, graph)
    }

    /// Launch `xdot.py`, coming from `x` keybinding.
    pub(crate) fn xdot(&mut self) -> DotViewerResult<SuccessState> {
        if !std::path::Path::new("./exports/current.dot").exists() {
            return Err(DotViewerError::XdotError);
        }

        let xdot = std::process::Command::new("xdot")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .arg("./exports/current.dot")
            .spawn();

        xdot.map(|_| SuccessState::XdotSuccess).map_err(|_| DotViewerError::XdotError)
    }

    pub(crate) fn set_normal_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    pub(crate) fn set_command_mode(&mut self) {
        self.input.clear();

        self.mode = Mode::Command;
    }

    pub(crate) fn set_search_mode(&mut self, smode: SearchMode) {
        self.input.clear();

        self.mode = Mode::Search(smode);

        let view = self.tabs.selected();

        view.matches = List::from_iter(Vec::new());
        view.prevs = List::from_iter(Vec::new());
        view.nexts = List::from_iter(Vec::new());
    }

    pub(crate) fn set_popup_mode(&mut self, pmode: PopupMode) {
        self.mode = Mode::Popup(pmode);
    }
}

fn write_graph(filename: String, graph: &Graph) -> DotViewerResult<SuccessState> {
    std::fs::create_dir_all("./exports")?;
    let mut file_export = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(format!("./exports/{filename}.dot"))?;
    graph.to_dot(&mut file_export)?;

    let mut file_current = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("./exports/current.dot")?;
    graph.to_dot(&mut file_current)?;

    Ok(SuccessState::ExportSuccess(filename))
}
