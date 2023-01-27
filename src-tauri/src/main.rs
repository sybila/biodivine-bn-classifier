#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use]
extern crate json;

use crate::bdt::{AttributeId, Bdt, BdtNodeId, Outcome};

use biodivine_hctl_model_checker::model_checking::get_extended_symbolic_graph;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::{BooleanNetwork, ModelAnnotation};
use biodivine_lib_param_bn::symbolic_async_graph::GraphColors;

use clap::Parser;
use json::JsonValue;
use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::Read;
use std::sync::Mutex;
use tauri::State;
use zip::ZipArchive;

pub mod bdt;
pub mod util;

/// Structure to collect CLI arguments.
#[derive(Parser)]
#[clap(about = "An interactive explorer of HCTL properties through decision trees.")]
struct Arguments {
    /// Path to a zip archive that contains results of the classification.
    results_archive: String,

    /// Path to the original model (in aeon format) that was used for the classification.
    model: String,
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
async fn auto_expand_tree(
    tree: State<'_, Mutex<Bdt>>,
    node_id: usize,
    depth: u32,
) -> Result<String, String> {
    if depth > 10 {
        return Err("Maximum allowed depth is 10.".to_string());
    }
    let mut tree = tree.lock().unwrap();
    let node_id: BdtNodeId = if let Some(node_id) = BdtNodeId::try_from(node_id, &tree) {
        node_id
    } else {
        return Err(format!("Invalid node id {node_id}."));
    };
    let changed = tree.auto_expand(node_id, depth);
    Ok(tree.to_json_partial(&changed).to_string())
}

#[tauri::command]
async fn get_decision_attributes(
    tree: State<'_, Mutex<Bdt>>,
    node_id: usize,
) -> Result<String, String> {
    let tree = tree.lock().unwrap();
    let node = BdtNodeId::try_from(node_id, &tree);
    let node = if let Some(node) = node {
        node
    } else {
        return Err(format!("Invalid node id {node_id}."));
    };

    Ok(tree.attribute_gains_json(node).to_string())
}

#[tauri::command]
async fn apply_decision_attribute(
    tree: State<'_, Mutex<Bdt>>,
    node_id: usize,
    attribute_id: usize,
) -> Result<String, String> {
    let mut tree = tree.lock().unwrap();
    let Some(node) = BdtNodeId::try_from(node_id, &tree) else {
        return Err(format!("Invalid node id {node_id}."));
    };
    let Some(attribute) = AttributeId::try_from(attribute_id, &tree) else {
        return Err(format!("Invalid attribute id {attribute_id}."));
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
        return Err(format!("Invalid node id {node_id}."));
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

fn extract_sorted_prop_names(aeon_str: &str) -> Vec<String> {
    let annotation = ModelAnnotation::from_model_string(aeon_str);

    let mut prop_names: Vec<String> = Vec::new();
    // properties are expected as:     #!dynamic_property: NAME: FORMULA
    let properties_node = annotation.get_child(&["dynamic_property"]).unwrap();
    for path in properties_node.children().keys() {
        prop_names.push(path.clone());
    }

    // sort them
    prop_names.sort();
    prop_names
}

fn main() {
    let args = Arguments::parse();

    // file with original input containing the model (with formulae as annotations)
    let model_path = args.model;
    // zip archive with classification results
    let results_archive = args.results_archive;

    let archive_file = File::open(results_archive).unwrap();
    let mut archive = ZipArchive::new(archive_file).unwrap();

    // load number of HCTL variables from computation metadata
    let mut metadata_file = archive.by_name("metadata.txt").unwrap();
    let mut buffer = String::new();
    metadata_file.read_to_string(&mut buffer).unwrap();
    let num_hctl_vars: u16 = buffer.parse::<u16>().unwrap();
    drop(metadata_file);

    // load the BN model and generate the extended STG
    let aeon_str = read_to_string(model_path.as_str()).unwrap();
    let bn = BooleanNetwork::try_from(aeon_str.as_str()).unwrap();
    let graph = get_extended_symbolic_graph(&bn, num_hctl_vars);

    // load the property names from model annotations (to later display them)
    let prop_names = extract_sorted_prop_names(aeon_str.as_str());

    // collect the classification outcomes (colored sets) from the individual BDD dumps
    let mut outcomes = HashMap::new();

    // iterate over all files by indices
    // only BDD dumps (individual files) and a report&metadata are expected in the archive (for now)
    for idx in 0..archive.len() {
        let mut entry = archive.by_index(idx).unwrap();
        let file_name = entry.name().to_string();
        if file_name == *"report.txt" || file_name == *"metadata.txt" {
            continue;
        }

        // read the raw BDD and convert into the color set
        let mut bdd_str = String::new();
        entry.read_to_string(&mut bdd_str).unwrap();
        let bdd = Bdd::from_string(bdd_str.as_str());
        let color_set = GraphColors::new(bdd, graph.symbolic_context());

        // only collect non-empty categories (in case some empty sets appear)
        if color_set.approx_cardinality() > 0. {
            // set ID is vector of 0/1 telling which props are satisfied in given class
            let set_id = file_name
                .strip_prefix("bdd_dump_")
                .unwrap()
                .strip_suffix(".txt")
                .unwrap();

            // set names combines sorted names of properties that are satisfied
            let mut set_name = String::new();
            let mut empty = true;
            for (i, indicator) in set_id.chars().enumerate() {
                if indicator == '1' {
                    if empty {
                        set_name.push_str(prop_names[i].as_str());
                    } else {
                        set_name.push_str(format!(", {}", prop_names[i]).as_str());
                    }
                    empty = false;
                }
            }
            if empty {
                set_name.push('-');
            }

            outcomes.insert(Outcome::from(set_name), color_set);
        }
    }

    let bdt = Bdt::new_from_graph(outcomes, &graph);
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
