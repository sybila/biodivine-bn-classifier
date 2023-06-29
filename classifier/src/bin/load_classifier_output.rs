//! Binary for testing/debugging the loading of classifier's output.

use biodivine_hctl_model_checker::model_checking::get_extended_symbolic_graph;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColors;
use biodivine_lib_param_bn::BooleanNetwork;

use clap::Parser;
use std::fs::File;
use std::io::Read;
use zip::ZipArchive;

/// Structure to collect CLI arguments
#[derive(Parser)]
#[clap()]
struct Arguments {
    /// Path to a file with BN model file in aeon format.
    model_path: String,

    /// Path to an existing zip archive with report and files with BDDs.
    results_archive: String,
}

/// Read the contents of a file from a zip archive into a string.
fn read_zip_file(reader: &mut ZipArchive<File>, file_name: &str) -> String {
    let mut contents = String::new();
    let mut file = reader.by_name(file_name).unwrap();
    file.read_to_string(&mut contents).unwrap();
    contents
}

/// Collect the results of classification, which are BDDs representing color sets.
///
/// The `results_archive` contains files with each dumped BDD. Moreover, excluding these BDD files,
/// the dir contains a report and a metadata file. Metadata file contains information regarding the
/// number of extended symbolic HCTL variables supported by the BDDs.
///
/// The file at `model_path` contains the original parametrized model that was used for the
/// classification.
pub fn load_classifier_output(
    results_archive: &str,
    model_path: &str,
) -> Vec<(String, GraphColors)> {
    // open the zip archive with results
    let archive_file = File::open(results_archive).unwrap();
    let mut archive = ZipArchive::new(archive_file).unwrap();

    // Load the number of HCTL variables from computation metadata.
    let metadata = read_zip_file(&mut archive, "metadata.txt");
    let num_hctl_vars: u16 = metadata.trim().parse().unwrap();

    // Load the BN model and generate extended symbolic graph.
    let bn = BooleanNetwork::try_from_file(model_path).unwrap();
    let graph = get_extended_symbolic_graph(&bn, num_hctl_vars).unwrap();

    // collect the colored sets from the BDD dumps together with their "names"
    let mut named_color_sets = Vec::new();

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
        named_color_sets.push((file, color_set));
    }

    named_color_sets
}

/// Wrapper function to invoke the loading process by feeding it with CLI arguments.
fn main() {
    let args = Arguments::parse();

    // load the color sets that represent the classification results
    let color_sets =
        load_classifier_output(args.results_archive.as_str(), args.model_path.as_str());

    for (name, color_set) in color_sets {
        println!("{}: {:.0}", name, color_set.approx_cardinality());
    }
}
