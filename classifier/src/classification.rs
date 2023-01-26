//! Components regarding the BN classification based on HCTL properties

use crate::load_input::extract_properties_from_aeon;
use crate::write_output::{write_class_report_and_dump_bdds, write_empty_report};

use biodivine_hctl_model_checker::model_checking::{
    collect_unique_hctl_vars, get_extended_symbolic_graph, model_check_trees,
};
use biodivine_hctl_model_checker::preprocessing::node::HctlTreeNode;
use biodivine_hctl_model_checker::preprocessing::parser::parse_hctl_formula;
use biodivine_hctl_model_checker::preprocessing::tokenizer::try_tokenize_formula;
use biodivine_hctl_model_checker::preprocessing::utils::check_props_and_rename_vars;

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, SymbolicAsyncGraph,
};
use biodivine_lib_param_bn::BooleanNetwork;

use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;

/// Parse formulae into syntax trees, and count maximal number of HCTL variables in a formula
fn parse_formulae_and_count_vars(
    bn: &BooleanNetwork,
    formulae: Vec<String>,
) -> Result<(Vec<HctlTreeNode>, usize), String> {
    let mut parsed_trees = Vec::new();
    let mut max_num_hctl_vars = 0;
    for formula in formulae {
        let tokens = try_tokenize_formula(formula)?;
        let tree = parse_hctl_formula(&tokens)?;

        let modified_tree = check_props_and_rename_vars(*tree, HashMap::new(), String::new(), bn)?;
        let num_hctl_vars = collect_unique_hctl_vars(modified_tree.clone(), HashSet::new()).len();

        parsed_trees.push(modified_tree);
        if num_hctl_vars > max_num_hctl_vars {
            max_num_hctl_vars = num_hctl_vars;
        }
    }
    Ok((parsed_trees, max_num_hctl_vars))
}

/// Combine all assertions into one conjunction formula
/// Empty strings are transformed into "true" literal
fn combine_assertions(formulae: Vec<String>) -> String {
    let mut conjunction = String::new();
    for formula in formulae {
        conjunction.push_str(format!("({}) & ", formula).as_str());
    }

    // this ensures that formula does not end with "&"
    // Moreover, even if there are no assertions, resulting formula won't be empty and will be
    // satisfied in all colors
    conjunction.push_str("true");

    conjunction
}

/// Return the set of colors for which ALL system states are contained in the given color-vertex
/// set (i.e., if the given relation is a result of model checking a property, get colors for which
/// the property holds universally in every state).
fn get_universal_colors(
    stg: &SymbolicAsyncGraph,
    colored_vertices: &GraphColoredVertices,
) -> GraphColors {
    let complement = stg.mk_unit_colored_vertices().minus(colored_vertices);
    stg.unit_colors().minus(&complement.colors())
}

/// Extract properties from name-property pairs.
fn extract_properties(named_props: Vec<(String, String)>) -> Vec<String> {
    let mut properties = Vec::new();
    for (_, prop) in named_props {
        properties.push(prop.clone());
    }
    properties
}

/// Perform the classification of Boolean networks based on given properties.
/// Takes a path to a file in `extended AEON` format containing a partially defined BN model
/// and 2 sets of HCTL formulae. Assertions are formulae that must be satisfied, and properties
/// are formulae used for classification.
///
/// First, colors satisfying all assertions are computed, and then the set of remaining colors is
/// decomposed into categories based on satisfied properties. One class = colors where the same set
/// of properties is satisfied (universally).
///
/// Report and BDDs representing resulting classes are generated into `output_zip` archive.
pub fn classify(output_zip: &str, input_path: &str) -> Result<(), String> {
    // TODO: allow caching between model-checking assertions and properties somehow

    // load the model and two sets of formulae (from model annotations)
    let aeon_str = read_to_string(input_path).unwrap();
    let bn = BooleanNetwork::try_from(aeon_str.as_str())?;
    let (assertions, mut named_properties) = extract_properties_from_aeon(aeon_str.as_str()).unwrap();
    println!("Loaded all inputs.");

    // sort the properties by their names, and extract them
    named_properties.sort_by(|(a1, _), (b1, _)| a1.cmp(b1));
    let properties: Vec<String> = extract_properties(named_properties.clone());

    println!("Evaluating assertions...");
    // combine all assertions into one formula
    let single_assertion = combine_assertions(assertions.clone());

    // preproc all formulae at once - parse, compute max num of vars
    // it is crucial to include all formulae when computing number of HCTL vars needed
    let mut all_formulae = properties.clone();
    all_formulae.push(single_assertion);
    let (mut all_trees, num_hctl_vars) = parse_formulae_and_count_vars(&bn, all_formulae)?;

    // instantiate extended STG with enough variables to evaluate all formulae
    let graph = get_extended_symbolic_graph(&bn, num_hctl_vars as u16);

    // compute the colors (universally) satisfying the combined assertion formula
    let assertion_tree = all_trees.pop().unwrap();
    let result_assertions = model_check_trees(vec![assertion_tree], &graph)?;
    let valid_colors = get_universal_colors(&graph, result_assertions.get(0).unwrap());
    println!("Assertions evaluated.");

    if valid_colors.is_empty() {
        println!("No color satisfies given assertions. Aborting.");
        write_empty_report(&assertions, output_zip);
        return Ok(());
    }

    // restrict the colors on the symbolic graph
    let graph = SymbolicAsyncGraph::with_custom_context(
        bn,
        graph.symbolic_context().clone(),
        valid_colors.as_bdd().clone(),
    )?;

    println!("Evaluating properties...");
    // model check all properties on restricted graph
    let results_properties = model_check_trees(all_trees, &graph)?;
    let colors_properties: Vec<GraphColors> = results_properties
        .iter()
        .map(|colored_vertices| get_universal_colors(&graph, colored_vertices))
        .collect();

    // do the classification while printing the report and dumping resulting BDDs
    println!("Classifying based on model-checking results...");
    write_class_report_and_dump_bdds(
        &assertions,
        valid_colors,
        &named_properties,
        &colors_properties,
        output_zip,
        num_hctl_vars,
    );
    println!("Output finished.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::classification::combine_assertions;

    #[test]
    /// Test combining of assertion formulae into one conjunction formula.
    fn test_assertion_formulae_merge() {
        let formula1 = "3{x}: @{x}: AX {x}".to_string();
        let formula2 = "false".to_string();
        let formula3 = "a & b".to_string();

        // empty vector should result in true constant
        assert_eq!(combine_assertions(Vec::new()), "true".to_string());

        // otherwise, result should be a conjunction ending with `& true`
        assert_eq!(
            combine_assertions(vec![formula1.clone(), formula2.clone()]),
            "(3{x}: @{x}: AX {x}) & (false) & true".to_string(),
        );
        assert_eq!(
            combine_assertions(vec![formula1, formula2, formula3]),
            "(3{x}: @{x}: AX {x}) & (false) & (a & b) & true".to_string(),
        )
    }
}
