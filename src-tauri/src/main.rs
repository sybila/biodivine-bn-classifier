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
use json::JsonValue;
use tauri::State;
use crate::bdt::{AttributeId, Bdt, BdtNodeId, Outcome};

pub mod bdt;
pub mod util;

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
async fn get_tree_precision(tree: State<'_, Mutex<Bdt>>) -> Result<String, String> {
    Ok(format!("{}", tree.lock().unwrap().get_precision()))
}

#[tauri::command]
async fn set_tree_precision(tree: State<'_, Mutex<Bdt>>, precision: u32) -> Result<(), String> {
    let mut tree = tree.lock().unwrap();
    tree.set_precision(precision);
    Ok(())
}

#[tauri::command]
async fn get_decision_tree(tree: State<'_, Mutex<Bdt>>) -> Result<String, String> {
    let tree = tree.lock().unwrap();
    Ok(tree.to_json().to_string())
}

#[tauri::command]
async fn auto_expand_tree(tree: State<'_, Mutex<Bdt>>, node_id: usize, depth: u32) -> Result<String, String> {
    if depth > 10 {
        return Err("Maximum allowed depth is 10.".to_string());
    }
    let mut tree = tree.lock().unwrap();
    let node_id: BdtNodeId = if let Some(node_id) = BdtNodeId::try_from(node_id, &tree) {
        node_id
    } else {
        return Err(format!("Invalid node id {}.", node_id));
    };
    let changed = tree.auto_expand(node_id, depth);
    Ok(tree.to_json_partial(&changed).to_string())
}

#[tauri::command]
async fn get_decision_attributes(tree: State<'_, Mutex<Bdt>>, node_id: usize) -> Result<String, String> {
    let tree = tree.lock().unwrap();
    let node = BdtNodeId::try_from(node_id, &tree);
    let node = if let Some(node) = node {
        node
    } else {
        return Err(format!("Invalid node id {}.", node_id));
    };

    Ok(tree.attribute_gains_json(node).to_string())
}

#[tauri::command]
async fn apply_decision_attribute(tree: State<'_, Mutex<Bdt>>, node_id: usize, attribute_id: usize) -> Result<String, String> {
    let mut tree = tree.lock().unwrap();
    let Some(node) = BdtNodeId::try_from(node_id, &tree) else {
        return Err(format!("Invalid node id {}.", node_id));
    };
    let Some(attribute) = AttributeId::try_from(attribute_id, &tree) else {
        return Err(format!("Invalid attribute id {}.", attribute_id));
    };
    if let Ok((left, right)) = tree.make_decision(node, attribute) {
        let changes = array![
                tree.node_to_json(node),
                tree.node_to_json(left),
                tree.node_to_json(right),
            ];
        Ok(changes.to_string())
    } else {
        Err("Invalid node or attribute id.".to_string())
    }
}

#[tauri::command]
async fn revert_decision(tree: State<'_, Mutex<Bdt>>, node_id: usize) -> Result<String, String> {
    let mut tree = tree.lock().unwrap();
    let Some(node) = BdtNodeId::try_from(node_id, &tree) else {
        return Err(format!("Invalid node id {}.", node_id));
    };
    let removed = tree.revert_decision(node);
    let removed = removed
        .into_iter()
        .map(|v| v.to_index())
        .collect::<Vec<_>>();
    let response = object! {
                "node": tree.node_to_json(node),
                "removed": JsonValue::from(removed)
        };
    Ok(response.to_string())
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
    for (class, set) in classification.export_result() {
        outcomes.insert(Outcome::from(format!("{}", class)), set);
    }
    let bdt = Bdt::new_from_graph(outcomes, &stg);
    println!("Found attractors: {}", attractors.len());
    tauri::Builder::default()
        .manage(Mutex::new(bdt))
        .invoke_handler(tauri::generate_handler![
            get_tree_precision,
            set_tree_precision,
            get_decision_tree,
            auto_expand_tree,
            get_decision_attributes,
            apply_decision_attribute,
            revert_decision
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
