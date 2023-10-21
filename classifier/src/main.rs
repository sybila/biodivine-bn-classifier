//! Tool for symbolic classification of BN models based on dynamic properties.
//!
//! Takes a path to a model in `.aeon` format containing a partially specified BN model and
//! two sets of HCTL formulae: required properties (assertions) that must be satisfied,
//! and classification properties that are used for classification. All formulae are given
//! in a form of model annotations.
//!
//! First, conjunction of assertions is model-checked, and then the set of remaining colors is
//! decomposed into categories based on the classification properties they satisfy.
//!

pub mod classification;
pub mod load_inputs;
pub mod write_output;

use crate::classification::classify;
use clap::Parser;
use std::path::Path;
use std::time::SystemTime;

/// Structure to collect CLI arguments
#[derive(Parser)]
#[clap(about = "Symbolic classifier for BN models based on dynamic properties.")]
struct Arguments {
    /// Path to a file in annotated `aeon` format containing a PSBN model and 2 sets
    /// of HCTL formulae.
    input_path: String,

    /// Path to a zip archive to which a report and BDD results will be dumped.
    #[clap(short, long, default_value = "classification_result.zip")]
    output_zip: String,
}

/// Wrapper function to invoke the classifier and feed it with CLI arguments.
fn main() {
    let start = SystemTime::now();

    let args = Arguments::parse();
    println!("Loading input files...");

    let input_path = args.input_path;
    let output_name = args.output_zip;

    // check if given input path is valid
    if !Path::new(input_path.as_str()).is_file() {
        println!("{input_path} is not valid file");
        return;
    }

    let classification_res = classify(input_path.as_str(), output_name.as_str());

    if classification_res.is_err() {
        println!(
            "Error during computation: {}",
            classification_res.err().unwrap()
        )
    }

    println!(
        "Total computation time: {}ms",
        start.elapsed().unwrap().as_millis()
    );
}
