use crate::app::utils::{
    list::StatefulList,
    trie::SearchTrie,
};
use dot_graph::{
    parser::parse,
    structs::Graph,
};

#[derive(Debug, Clone)]
pub enum Mode {
    Traverse,
    Search,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Focus {
    Current,
    Prevs,
    Nexts,
}

pub struct App {
    pub quit: bool,
    pub mode: Mode,

    pub input: String, 
    pub errormsg: Option<String>,
    pub history: Vec<String>,

    pub lists: Lists,
}

pub struct Lists {
    pub graph: Graph,
    pub trie: SearchTrie,

    pub focus: Focus,
    pub current: StatefulList<String>,
    pub prevs: StatefulList<String>,
    pub nexts: StatefulList<String>,
    pub search: StatefulList<String>,
}

impl App {
    pub fn new(path: &str) -> App {                
        App {
            quit: false,
            mode: Mode::Traverse,
            input: String::from(""),
            history: Vec::new(),
            errormsg: None,
            lists: Lists::new(path),
        }
    }
}

impl Lists {
    pub fn new(path: &str) -> Lists {
        let graph = parse(path); 
        let nodes: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();  
        let trie = SearchTrie::new(&nodes);

        let mut lists = Lists {
            graph,
            trie,
            focus: Focus::Current,
            current: StatefulList::with_items(nodes),
            prevs: StatefulList::with_items(Vec::new()),
            nexts: StatefulList::with_items(Vec::new()),
            search: StatefulList::with_items(Vec::new()),
        };

        lists.update();

        lists
    }

    pub fn current(&self) -> Option<String> {
        self.current.selected()
    }

    pub fn idx(&self) -> Option<usize> {
        self.current.state.selected()
    }

    pub fn count(&self) -> usize {
        self.current.items.len()
    }

    pub fn update(&mut self) {
        let id = self.current().unwrap();

        let prevs = self.graph.froms(&id).iter().map(|n| n.to_string()).collect();
        self.prevs = StatefulList::with_items(prevs);

        let nexts = self.graph.tos(&id).iter().map(|n| n.to_string()).collect();
        self.nexts = StatefulList::with_items(nexts);
    }
}
