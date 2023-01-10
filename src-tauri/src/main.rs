#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
#[macro_use]
extern crate json;

use std::collections::HashMap;
use std::sync::Mutex;
use biodivine_aeon_server::scc::Classifier;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use tauri::State;
use crate::bdt::{Bdt, Outcome};

pub mod bdt;
pub mod util;

#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello, {}!", name)
}

fn attractors(stg: &SymbolicAsyncGraph, set: &GraphColoredVertices) -> Vec<GraphColoredVertices> {
    let mut results: Vec<GraphColoredVertices> = Vec::new();
    let root_stg = stg;

    // Restricted STG containing only the remaining vertices.
    let mut active_stg = root_stg.restrict(set);
    while !active_stg.unit_colored_vertices().is_empty() {
        // Pick a (colored) vertex and compute the backward-reachable basin.
        let pivot = active_stg.unit_colored_vertices().pick_vertex();
        let pivot_basin = active_stg.reach_backward(&pivot);
        let pivot_fwd = active_stg.reach_forward(&pivot);

        let scc = pivot_fwd.intersect(&pivot_basin);
        let non_terminal = pivot_fwd.minus(&pivot_basin).colors();
        let bottom = scc.minus_colors(&non_terminal);

        // If there is something remaining in the pivot component, report it as attractor.
        if !bottom.is_empty() {
            results.push(bottom);
        }

        // Further restrict the STG by removing the current basin.
        active_stg = active_stg.restrict(&active_stg.unit_colored_vertices().minus(&pivot_basin));
    }
    return results;
}

#[tauri::command]
fn get_tree_precision(tree: State<Mutex<Bdt>>) -> String {
    format!("{}", tree.lock().unwrap().get_precision())
}

#[tauri::command]
fn set_tree_precision(tree: State<Mutex<Bdt>>, precision: u32) {
    let mut tree = tree.lock().unwrap();
    tree.set_precision(precision)
}

fn main() {
    let model = BooleanNetwork::try_from(std::fs::read_to_string("model.aeon").unwrap().as_str()).unwrap();
    let stg = SymbolicAsyncGraph::new(model).unwrap();
    println!("Start attractors.");
    let attractors = attractors(&stg, stg.unit_colored_vertices());
    let classification = Classifier::new(&stg);
    for attractor in &attractors {
        classification.add_component(attractor.clone(), &stg);
    }
    let mut outcomes = HashMap::new();
    for (i, (c, _)) in classification.export_components().iter().enumerate() {
        outcomes.insert(Outcome::from(i), c.colors());
    }
    let bdt = Bdt::new(outcomes, Vec::new());
    println!("Found attractors: {}", attractors.len());
    tauri::Builder::default()
        .manage(Mutex::new(bdt))
        .invoke_handler(tauri::generate_handler![greet, get_tree_precision, set_tree_precision])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
