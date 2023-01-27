//! Tool for symbolic classification of BN models based on dynamic properties.
//!
//! Takes a path to a model in aeon format containing a partially defined BN model and
//! two sets of HCTL formulae - assertions that must be satisfied, and properties that are
//! used for classification. All formulae are given in a form of model annotations.
//!
//! First, conjunction of assertions is model-checked, and then the set of remaining colors is
//! decomposed into categories based on the properties they satisfy.
//!

pub mod classification;
pub mod write_output;

use crate::classification::classify;
use clap::Parser;
use std::path::Path;

/// Structure to collect CLI arguments
#[derive(Parser)]
#[clap(about = "Symbolic classifier for BN models based on dynamic properties.")]
struct Arguments {
    /// Path to a file in annotated `aeon` format containing a BN model and 2 sets of HCTL formulae.
    input_path: String,

    /// Path to a zip archive to which a report and BDD results will be dumped.
    #[clap(short, long, default_value = "classification_result.zip")]
    output_zip: String,
}

/// Wrapper function to invoke the classifier and feed it with CLI arguments.
fn main() {
    let args = Arguments::parse();
    println!("Loading input files...");

    let input_path = args.input_path;
    let output_name = args.output_zip;

    // check if given input path is valid
    if !Path::new(input_path.as_str()).is_file() {
        println!("{} is not valid file", input_path);
        return;
    }

    let classification_res = classify(output_name.as_str(), input_path.as_str());

    if classification_res.is_err() {
        println!(
            "Error during computation: {}",
            classification_res.err().unwrap()
        )
    }
}