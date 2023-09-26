#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use]
extern crate json;

use crate::bdt::{AttributeId, Bdt, BdtNodeId, Outcome};

use biodivine_lib_bdd::{Bdd, BddPartialValuation};
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};
use biodivine_lib_param_bn::{BooleanNetwork, ModelAnnotation, RegulatoryGraph};

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use clap::Parser;
use json::JsonValue;
use rand::prelude::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::ops::DerefMut;
use std::path::Path;
use std::sync::Mutex;
use tauri::api::dialog;
use tauri::{AppHandle, State};
use zip::write::{FileOptions, ZipWriter};
use zip::ZipArchive;

pub mod bdt;
pub mod util;

/// Structure to collect CLI arguments.
#[derive(Parser)]
#[clap(about = "An interactive explorer of HCTL properties through decision trees.")]
struct Arguments {
    /// Path to a zip archive that contains results of the classification.
    results_archive: Option<String>,
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

/// Get number of networks represented by the node.
#[tauri::command]
async fn get_num_node_networks(
    tree: State<'_, Mutex<Bdt>>,
    node_id: usize,
) -> Result<String, String> {
    let tree = tree.lock().unwrap();
    let Some(node_id) = BdtNodeId::try_from(node_id, &tree) else {
        return Err(format!("Invalid node id {node_id}."));
    };
    let cardinality = tree.all_node_params(node_id).exact_cardinality();
    Ok(format!("{cardinality}"))
}

/// Get all named properties that were used for classification.
#[tauri::command]
async fn get_all_named_properties(
    tree: State<'_, Mutex<Bdt>>,
) -> Result<HashMap<String, String>, String> {
    let tree = tree.lock().unwrap();
    Ok(tree.properties().clone())
}

/// Get universally satisfied properties in given node.
#[tauri::command]
async fn get_node_universal_sat_props(
    tree: State<'_, Mutex<Bdt>>,
    node_id: usize,
) -> Result<Vec<String>, String> {
    let tree = tree.lock().unwrap();
    let Some(node_id) = BdtNodeId::try_from(node_id, &tree) else {
        return Err(format!("Invalid node id {node_id}."));
    };
    let sat_props = tree.node_universal_sat_props(node_id);
    let mut sat_props = Vec::from_iter(sat_props.iter().map(|it| it.to_string()));
    sat_props.sort();
    Ok(sat_props)
}

/// Get universally unsatisfied properties in given node.
#[tauri::command]
async fn get_node_universal_unsat_props(
    tree: State<'_, Mutex<Bdt>>,
    node_id: usize,
) -> Result<Vec<String>, String> {
    let tree = tree.lock().unwrap();
    let Some(node_id) = BdtNodeId::try_from(node_id, &tree) else {
        return Err(format!("Invalid node id {node_id}."));
    };
    let unsat_props = tree.node_universal_unsat_props(node_id);
    let mut unsat_props = Vec::from_iter(unsat_props.iter().map(|it| it.to_string()));
    unsat_props.sort();
    Ok(unsat_props)
}

#[tauri::command]
async fn save_file(path: &str, content: &str) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| format!("{e:?}"))
}

/// Create a zip archive containing multiple AEON witness networks.
#[tauri::command]
async fn save_zip_archive(path: &str, list_file_contents: Vec<&str>) -> Result<(), String> {
    // Prepare the archive first
    let archive_path = Path::new(path);
    // If there are some non existing dirs in path, create them.
    let prefix = archive_path.parent().unwrap();
    std::fs::create_dir_all(prefix).map_err(|e| format!("{e:?}"))?;
    // Create a zip writer for the desired archive.
    let archive = File::create(archive_path).map_err(|e| format!("{e:?}"))?;
    let mut zip_writer = ZipWriter::new(archive);

    for (i, file_content) in list_file_contents.iter().enumerate() {
        zip_writer
            .start_file(format!("witness_{i}.aeon"), FileOptions::default())
            .map_err(|e| format!("{e:?}"))?;
        writeln!(zip_writer, "{file_content}").map_err(|e| format!("{e:?}"))?;
    }

    zip_writer.finish().map_err(|e| format!("{e:?}"))?;
    Ok(())
}

/// Randomly select a color from the given set of colors.
/// This is a workaround that should be modified in the future.
pub fn pick_random_color(
    rng: &mut StdRng,
    graph: &SymbolicAsyncGraph,
    color_set: &GraphColors,
) -> GraphColors {
    let ctx = graph.symbolic_context();
    let random_witness = color_set.as_bdd().random_valuation(rng).unwrap();
    let mut partial_valuation = BddPartialValuation::empty();
    for var in ctx.parameter_variables() {
        partial_valuation.set_value(*var, random_witness[*var]);
    }
    let singleton_bdd = ctx
        .bdd_variable_set()
        .mk_conjunctive_clause(&partial_valuation);
    // We can use the "raw copy" function because into the new BDD, we only carried over
    // the BDD variables that encode network parameters.
    color_set.copy(singleton_bdd)
}

#[tauri::command]
async fn download_witnesses(
    tree: State<'_, Mutex<Bdt>>,
    graph: State<'_, Mutex<SymbolicAsyncGraph>>,
    path: &str,
    node_id: usize,
    witness_count: usize,
    seed: Option<u64>,
) -> Result<(), String> {
    // Prepare the archive first
    let archive_path = Path::new(path);
    // If there are some non existing dirs in path, create them.
    let prefix = archive_path.parent().unwrap();
    std::fs::create_dir_all(prefix).map_err(|e| format!("{e:?}"))?;
    // Create a zip writer for the desired archive.
    let archive = File::create(archive_path).map_err(|e| format!("{e:?}"))?;
    let mut zip_writer = ZipWriter::new(archive);

    let tree = tree.lock().unwrap();
    let graph = graph.lock().unwrap();
    let Some(node_id) = BdtNodeId::try_from(node_id, &tree) else {
        return Err(format!("Invalid node id {node_id}."));
    };

    let mut node_colors = tree.all_node_params(node_id);
    let mut i = 0;

    let mut random_state = seed.map(StdRng::seed_from_u64);

    // collect `num_witnesses` networks
    while i < witness_count && !node_colors.is_empty() {
        // get singleton color for the witness
        let witness_color = if let Some(std_rng) = random_state.as_mut() {
            // For random networks, we need to be a bit more creative... (although, support for
            // this in lib-param-bn would be nice).
            pick_random_color(std_rng, &graph, &node_colors)
        } else {
            // The `SymbolicAsyncGraph::pick_singleton` should be deterministic.
            node_colors.pick_singleton()
        };
        assert!(witness_color.is_singleton());

        // remove the color from the set
        node_colors = node_colors.minus(&witness_color);
        i += 1;

        // Write the network into the zip.
        let file_content = graph.pick_witness(&witness_color).to_string();
        zip_writer
            .start_file(format!("witness_{i}.aeon"), FileOptions::default())
            .map_err(|e| format!("{e:?}"))?;
        writeln!(zip_writer, "{file_content}").map_err(|e| format!("{e:?}"))?;
    }

    zip_writer.finish().map_err(|e| format!("{e:?}"))?;
    Ok(())
}

/// Wrapper to only get a single witness
#[tauri::command]
async fn get_witness(
    tree: State<'_, Mutex<Bdt>>,
    graph: State<'_, Mutex<SymbolicAsyncGraph>>,
    random_state: State<'_, Mutex<StdRng>>,
    node_id: usize,
    randomize: bool,
) -> Result<String, String> {
    let singleton_witness =
        get_n_witnesses(tree, graph, random_state, 1, node_id, randomize).await?;
    assert_eq!(singleton_witness.len(), 1);
    Ok(singleton_witness.into_iter().next().unwrap())
}

#[tauri::command]
async fn get_n_witnesses(
    tree: State<'_, Mutex<Bdt>>,
    graph: State<'_, Mutex<SymbolicAsyncGraph>>,
    random_state: State<'_, Mutex<StdRng>>,
    num_witnesses: i32,
    node_id: usize,
    randomize: bool,
) -> Result<Vec<String>, String> {
    let tree = tree.lock().unwrap();
    let graph = graph.lock().unwrap();
    let Some(node_id) = BdtNodeId::try_from(node_id, &tree) else {
        return Err(format!("Invalid node id {node_id}."));
    };

    let mut node_colors = tree.all_node_params(node_id);
    let mut witnesses_bns: Vec<BooleanNetwork> = Vec::new();
    let mut i = 0;

    // just to make it explicit (this condition is also checked before this function is called)
    assert!((num_witnesses as f64) <= node_colors.approx_cardinality());

    // collect `num_witnesses` networks
    while i < num_witnesses && !node_colors.is_empty() {
        // get singleton color for the witness
        let witness_color = if !randomize {
            // The `SymbolicAsyncGraph::pick_singleton` should be deterministic.
            node_colors.pick_singleton()
        } else {
            // For random networks, we need to be a bit more creative... (although, support for
            // this in lib-param-bn would be nice).
            let mut generator = random_state.lock().unwrap();
            let std_rng: &mut StdRng = generator.deref_mut();
            pick_random_color(std_rng, &graph, &node_colors)
        };
        assert!(witness_color.is_singleton());
        witnesses_bns.push(graph.pick_witness(&witness_color));

        // remove the color from the set
        node_colors = node_colors.minus(&witness_color);
        i += 1;
    }

    Ok(witnesses_bns.into_iter().map(|it| it.to_string()).collect())
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

/// A command which completely replaces the current graph/tree state
/// based on a new archive file chosen by the user.
///
/// Note that it is up to the GUI to ensure that visible part of the tree is actually
/// updated properly after this call is finished. Typically, this is done through
/// `loadBifurcationTree` in `main.js`.
#[tauri::command]
async fn reload_tree(
    app: AppHandle,
    tree: State<'_, Mutex<Bdt>>,
    graph: State<'_, Mutex<SymbolicAsyncGraph>>,
) -> Result<(), ()> {
    let path = dialog::blocking::FileDialogBuilder::new()
        .set_title("Pick bn-classifier result archive.")
        .add_filter("zip", &["zip"])
        .pick_file();
    let Some(path) = path else {
        // Exit silently if no path was chosen.
        app.exit(2);
        return Ok(());
    };
    let path = path.to_str().unwrap().to_string();
    let (g, t) = setup_environment(path.as_str());
    let mut tree = tree.lock().unwrap();
    *tree = t;
    let mut graph = graph.lock().unwrap();
    *graph = g;
    Ok(())
}

/// Read the list of named properties from an `.aeon` model annotation object.
///
/// The properties are expected to appear as `#!dynamic_property: NAME: FORMULA` model annotations.
/// They are returned in alphabetic order w.r.t. the property name.
fn read_model_properties(annotations: &ModelAnnotation) -> Result<Vec<(String, String)>, String> {
    let Some(property_node) = annotations.get_child(&["dynamic_property"]) else {
        return Ok(Vec::new());
    };
    let mut properties = Vec::with_capacity(property_node.children().len());
    for (name, child) in property_node.children() {
        if !child.children().is_empty() {
            // TODO:
            //  This might actually be a valid (if ugly) way for adding extra meta-data to
            //  properties, but let's forbid it for now and we can enable it later if
            //  there is an actual use for it.
            return Err(format!("Property `{name}` contains nested values."));
        }
        let Some(value) = child.value() else {
            return Err(format!("Found empty dynamic property `{name}`."));
        };
        if value.lines().count() > 1 {
            return Err(format!("Found multiple properties named `{name}`."));
        }
        properties.push((name.clone(), value.clone()));
    }
    // Sort alphabetically to avoid possible non-determinism down the line.
    properties.sort_by(|(x, _), (y, _)| x.cmp(y));
    Ok(properties)
}

/// Read the contents of a file from a zip archive into a string.
fn read_zip_file(reader: &mut ZipArchive<File>, file_name: &str) -> String {
    let mut contents = String::new();
    let mut file = reader.by_name(file_name).unwrap();
    file.read_to_string(&mut contents).unwrap();
    contents
}

/// Load a classification archive from the given path into a valid symbolic graph
/// and decision tree.
///
/// This can happen either before the application starts (if path is given as CLI arg),
/// or after the first window is opened (if no path is given).
fn setup_environment(archive_path: &str) -> (SymbolicAsyncGraph, Bdt) {
    // Open the zip archive with classification results.
    let archive_file = File::open(archive_path).unwrap();
    let mut archive = ZipArchive::new(archive_file).unwrap();

    // Load the BN model (from the archive) and generate the extended STG.
    let aeon_str = read_zip_file(&mut archive, "model.aeon");
    let bn = BooleanNetwork::try_from(aeon_str.as_str()).unwrap();
    let graph = SymbolicAsyncGraph::new(bn).unwrap();

    // load the property names from model annotations (to later display them)
    let annotations = ModelAnnotation::from_model_string(aeon_str.as_str());
    let properties = read_model_properties(&annotations).unwrap();
    let properties_map = HashMap::from_iter(properties.iter().cloned());

    // collect the classification outcomes (colored sets) from the individual BDD dumps
    let mut outcomes = HashMap::new();

    // Load all class BDDs from files in the archive.
    let files = archive
        .file_names()
        .map(|it| it.to_string())
        .collect::<Vec<_>>();

    for file in files {
        if !file.starts_with("bdd_dump_") {
            // Only read BDD dumps.
            continue;
        }

        let bdd_string = read_zip_file(&mut archive, file.as_str());
        let bdd = Bdd::from_string(bdd_string.as_str());
        let color_set = GraphColors::new(bdd, graph.symbolic_context());

        let outcome_id = file.strip_prefix("bdd_dump_").unwrap();
        let outcome_id = outcome_id.strip_suffix(".txt").unwrap();

        let outcome = if properties_map.is_empty() {
            // This is "custom" archive that does not hold HCTL properties, in which case,
            // we want to display them simply through their IDs, since we don't have the
            // property.
            Outcome::from(outcome_id.to_string())
        } else {
            let mut valid_properties = Vec::new();
            for ((name, _formula), is_valid) in properties.iter().zip(outcome_id.chars()) {
                if is_valid == '1' {
                    valid_properties.push(name.clone());
                }
            }

            let outcome_name = if valid_properties.is_empty() {
                "-".to_string()
            } else {
                valid_properties.join(", ")
            };

            Outcome::from(outcome_name)
        };
        // The insert should create a new item, otherwise the archive is malformed.
        assert!(outcomes.insert(outcome, color_set).is_none());
    }

    let bdt = Bdt::new_from_graph(outcomes, &graph, properties_map);
    (graph, bdt)
}

fn main() {
    // First, read model and classification archive, then build a tree from said archive,
    // finally, open GUI with the tree.

    let args = Arguments::parse();

    let (graph, bdt) = if let Some(user_path) = args.results_archive {
        setup_environment(user_path.as_str())
    } else {
        // Make a fake Boolean network and a fake decision tree. The GUI will force the user
        // to reload the tree once the window has started.
        let bn = BooleanNetwork::new(RegulatoryGraph::new(vec!["x".to_string()]));
        let graph = SymbolicAsyncGraph::new(bn).unwrap();
        let tree = Bdt::new(HashMap::new(), Vec::new(), HashMap::new());
        (graph, tree)
    };

    let random_state = StdRng::seed_from_u64(1234567890);
    tauri::Builder::default()
        .manage(Mutex::new(bdt))
        .manage(Mutex::new(graph))
        .manage(Mutex::new(random_state))
        .invoke_handler(tauri::generate_handler![
            get_tree_precision,
            set_tree_precision,
            get_decision_tree,
            get_num_node_networks,
            download_witnesses,
            get_node_universal_sat_props,
            get_node_universal_unsat_props,
            get_all_named_properties,
            auto_expand_tree,
            get_decision_attributes,
            apply_decision_attribute,
            revert_decision,
            get_witness,
            get_n_witnesses,
            save_file,
            save_zip_archive,
            reload_tree,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
