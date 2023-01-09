mod parser;
mod graph;
mod command;
mod context;
mod error;

use std::env;
use std::fs;
use crate::context::Context;
use repl_rs::{ Command, Parameter, Result, Repl };

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    // parse graph.dot in current directory
    let dot = fs::read_to_string(filename).expect("no such file");
    let graph = parser::parse(&dot);
    
    // define repl commands
    let mut repl = Repl::new(Context { filename: filename.clone(), graph: graph.clone(), center: graph.nodes.first().unwrap().clone() ,depth_limit: 1 })
        .with_name("dot-viewer")
        .with_version("dev")
        .add_command(
            Command::new("show", command::show)
                .with_help("Show graph centered at current node"),
        )
        .add_command(
            Command::new("export", command::export)
                .with_parameter(Parameter::new("filename").set_required(true)?)?
                .with_help("Export graph centered at current node to dot"),
        )
        .add_command(
            Command::new("render", command::render)
                .with_parameter(Parameter::new("all").set_required(false)?)?
                .with_help("Render graph centered at current node with xdot, or complete graph given \"all\" option"),
        )
        .add_command(
            Command::new("goto", command::goto)
                .with_parameter(Parameter::new("node").set_required(true)?)?
                .with_help("Go to a node in graph"),
        )
        .add_command(
            Command::new("depth", command::depth)
                .with_parameter(Parameter::new("depth").set_required(true)?)?
                .with_help("Set visualization depth"),
        );

    // run repl
    repl.run()
}
