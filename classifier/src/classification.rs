//! Components regarding the BN classification based on HCTL properties

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
use biodivine_lib_param_bn::{BooleanNetwork, ModelAnnotation};

use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;

type NamedFormulaeVec = Vec<(String, String)>;

/// Get the HCTL formulae from annotations of aeon model.
/// Assertion formulae are expected in a form:   #!dynamic_assertion: FORMULA
/// Properties are expected in a form:           #!dynamic_property: NAME: FORMULA
///
/// Return set of annotations (in order) and set of named properties (in random order).
pub fn extract_formulae_from_aeon(
    aeon_string: &str,
) -> Result<(Vec<String>, NamedFormulaeVec), String> {
    let annotation = ModelAnnotation::from_model_string(aeon_string);

    let mut assertions: Vec<String> = Vec::new();
    // assertions are expected as:     #!dynamic_assertion: FORMULA
    if let Some(assertions_node) = annotation.get_child(&["dynamic_assertion"]) {
        for value in assertions_node.value().unwrap().as_str().lines() {
            assertions.push(value.to_string())
        }
    }

    let mut named_properties: NamedFormulaeVec = Vec::new();
    // properties are expected as:     #!dynamic_property: NAME: FORMULA
    if let Some(properties_node) = annotation.get_child(&["dynamic_property"]) {
        for (path, child) in properties_node.children() {
            if let Some(property) = child.value() {
                if property.lines().count() > 1 {
                    return Err("Properties cannot share names.".to_string());
                }
                named_properties.push((path.clone(), property.clone()))
            } else {
                return Err("Property annotation can't be that nested.".to_string());
            }
        }
    }

    Ok((assertions, named_properties))
}

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
        conjunction.push_str(format!("({formula}) & ").as_str());
    }

    // If there are no assertions, resulting formula won't be empty and will be satisfied by
    // all colors (moreover, it ensures that formula does not end with "&")
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
pub fn extract_properties(named_props: NamedFormulaeVec) -> Vec<String> {
    let mut properties = Vec::new();
    for (_, prop) in named_props {
        properties.push(prop.clone());
    }
    properties
}

/// Perform the classification of Boolean networks based on given properties.
/// Takes a path to a file in annotated `AEON` format containing a partially defined BN model
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
    let (assertions, mut named_properties) = extract_formulae_from_aeon(aeon_str.as_str()).unwrap();
    println!("Loaded all inputs.");

    // sort the properties by their names, and extract them
    named_properties.sort_by(|(a1, _), (b1, _)| a1.cmp(b1));
    let properties: Vec<String> = extract_properties(named_properties.clone());

    println!("Parsing formulae and generating model representation...");
    // combine all assertions into one formula
    let single_assertion = combine_assertions(assertions.clone());

    // preproc all formulae at once - parse, compute max num of vars
    // it is crucial to include all formulae when computing number of HCTL vars needed
    let mut all_formulae = properties.clone();
    all_formulae.push(single_assertion);
    let (mut all_trees, num_hctl_vars) = parse_formulae_and_count_vars(&bn, all_formulae)?;
    println!(
        "Successfully parsed {} assertions and {} properties.",
        assertions.len(),
        properties.len(),
    );

    // instantiate extended STG with enough variables to evaluate all formulae
    let graph = get_extended_symbolic_graph(&bn, num_hctl_vars as u16);
    println!(
        "Successfully generated model with {} vars and {} params.",
        graph.symbolic_context().num_state_variables(),
        graph.symbolic_context().num_parameter_variables(),
    );

    println!("Evaluating assertions...");
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
    use crate::classification::{
        combine_assertions, extract_formulae_from_aeon, extract_properties,
        parse_formulae_and_count_vars,
    };
    use biodivine_lib_param_bn::BooleanNetwork;

    #[test]
    /// Test the formulae parsing and variable counting
    fn test_formulae_variable_count() {
        let aeon_str = r"
            $v_2:!v_3
            v_3 -| v_2
            $v_3:v_3
            v_3 -> v_3
        ";
        let bn = BooleanNetwork::try_from(aeon_str).unwrap();
        let formulae = vec![
            "!{x}: AX {x}".to_string(),
            "!{y}: (AG EF {y} & (!{z}: AX {z}))".to_string(),
        ];

        let (_, var_count) = parse_formulae_and_count_vars(&bn, formulae).unwrap();
        assert_eq!(var_count, 2);
    }

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

    #[test]
    /// Test extracting the formulae from the AEON format annotations.
    fn test_extracting_formulae() {
        let aeon_str = r"
            #! dynamic_assertion: #`true`#
            #! dynamic_assertion: #`3{x}: @{x}: AX {x}`#
            #! dynamic_property: p1: #`3{x}: @{x}: AG EF {x}`#
            #! dynamic_property: p2: #`3{x}: @{x}: AX AF {x}`#
            $v_2:!v_3
            v_3 -| v_2
            $v_3:v_3
            v_3 -> v_3
        ";

        let (assertions, named_properties) = extract_formulae_from_aeon(aeon_str).unwrap();

        assert_eq!(
            assertions,
            vec!["true".to_string(), "3{x}: @{x}: AX {x}".to_string()]
        );

        assert_eq!(named_properties.len(), 2);
        assert!(named_properties.contains(&("p1".to_string(), "3{x}: @{x}: AG EF {x}".to_string())));
        assert!(named_properties.contains(&("p2".to_string(), "3{x}: @{x}: AX AF {x}".to_string())));
    }

    #[test]
    /// Test that extracting entities from the corrupted AEON format annotations.
    fn test_extracting_formulae_corrupted() {
        let aeon_str = r"
            #! dynamic_assertion: #`true`#
            #! dynamic_property: p1: #`3{x}: @{x}: AG EF {x}`#
            #! dynamic_property: p1: #`3{x}: @{x}: AX {x}`#
            $v_2:!v_3
            v_3 -| v_2
            $v_3:v_3
            v_3 -> v_3
        ";
        assert!(extract_formulae_from_aeon(aeon_str).is_err());
        assert_eq!(
            extract_formulae_from_aeon(aeon_str).err().unwrap(),
            "Properties cannot share names.".to_string()
        );
    }

    #[test]
    /// Test extracting properties from name-property pairs.
    fn test_extract_properties() {
        let named_props = vec![
            ("p1".to_string(), "true".to_string()),
            ("p2".to_string(), "false".to_string()),
        ];
        assert_eq!(
            extract_properties(named_props),
            vec!["true".to_string(), "false".to_string()]
        );
    }
}
