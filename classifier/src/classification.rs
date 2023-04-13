//! Components regarding the BN classification based on HCTL properties

use crate::write_output::{write_class_report_and_dump_bdds, write_empty_report};
use std::cmp::max;

use biodivine_hctl_model_checker::model_checking::{
    collect_unique_hctl_vars, get_extended_symbolic_graph, model_check_tree, model_check_trees,
};
use biodivine_hctl_model_checker::preprocessing::node::HctlTreeNode;
use biodivine_hctl_model_checker::preprocessing::parser::parse_and_minimize_hctl_formula;

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, SymbolicAsyncGraph,
};
use biodivine_lib_param_bn::{BooleanNetwork, ModelAnnotation};

use std::collections::HashSet;

type NamedFormulaeVec = Vec<(String, String)>;

/// Read the list of assertions from an `.aeon` model annotation object.
///
/// The assertions are expected to appear as `#!dynamic_assertion: FORMULA` model annotations
/// and they are returned in declaration order.
fn read_model_assertions(annotations: &ModelAnnotation) -> Vec<String> {
    let Some(list) = annotations.get_value(&["dynamic_assertion"]) else {
        return Vec::new();
    };
    list.lines().map(|it| it.to_string()).collect()
}

/// Read the list of named properties from an `.aeon` model annotation object.
///
/// The properties are expected to appear as `#!dynamic_property: NAME: FORMULA` model annotations.
/// They are returned in alphabetic order w.r.t. the property name.
fn read_model_properties(annotations: &ModelAnnotation) -> Result<NamedFormulaeVec, String> {
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

/// Combine all HCTL assertions in the given list into a single conjunction of assertions.
fn build_combined_assertion(assertions: &[String]) -> String {
    if assertions.is_empty() {
        "true".to_string()
    } else {
        // Add parenthesis to each assertion.
        let assertions: Vec<String> = assertions.iter().map(|it| format!("({it})")).collect();
        // Join them into one big conjunction.
        assertions.join(" & ")
    }
}

/// Return the set of colors for which ALL system states are contained in the given color-vertex
/// set (i.e., if the given relation is a result of model checking a property, get colors for which
/// the property holds universally in every state).
///
/// Formally, this is a universal projection on the colors of the given `colored_vertices`.
fn get_universal_colors(
    stg: &SymbolicAsyncGraph,
    colored_vertices: &GraphColoredVertices,
) -> GraphColors {
    let complement = stg.unit_colored_vertices().minus(colored_vertices);
    stg.unit_colors().minus(&complement.colors())
}

/// Extract properties from name-property pairs.
pub fn extract_properties(named_props: &NamedFormulaeVec) -> Vec<String> {
    named_props.iter().map(|(_, x)| x.clone()).collect()
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
    let Ok(aeon_str) = std::fs::read_to_string(input_path) else {
        return Err(format!("Input file `{input_path}` is not accessible."));
    };
    let bn = BooleanNetwork::try_from(aeon_str.as_str())?;
    let annotations = ModelAnnotation::from_model_string(aeon_str.as_str());
    let assertions = read_model_assertions(&annotations);
    let named_properties = read_model_properties(&annotations)?;
    println!("Loaded all inputs.");

    println!("Parsing formulae and generating model representation...");
    // Combine all assertions into one formula and add it to the list of properties.
    let assertion = build_combined_assertion(&assertions);
    println!(
        "Successfully parsed {} assertions and {} properties.",
        assertions.len(),
        named_properties.len(),
    );

    // Parse all formulae and count the max. number of HCTL variables across formulae.
    let assertion_tree = parse_and_minimize_hctl_formula(&bn, &assertion)?;
    let mut num_hctl_vars = collect_unique_hctl_vars(assertion_tree.clone(), HashSet::new()).len();
    let mut property_trees: Vec<HctlTreeNode> = Vec::new();
    for (_name, formula) in &named_properties {
        let tree = parse_and_minimize_hctl_formula(&bn, formula.as_str())?;
        let tree_vars = collect_unique_hctl_vars(tree.clone(), HashSet::new()).len();
        num_hctl_vars = max(num_hctl_vars, tree_vars);
        property_trees.push(tree);
    }

    // Instantiate extended STG with enough variables to evaluate all formulae.
    let graph = get_extended_symbolic_graph(&bn, num_hctl_vars as u16);
    println!(
        "Successfully generated model with {} vars and {} params.",
        graph.symbolic_context().num_state_variables(),
        graph.symbolic_context().num_parameter_variables(),
    );

    println!("Evaluating assertions...");
    // Compute the colors (universally) satisfying the combined assertion formula.
    let assertion_result = model_check_tree(assertion_tree, &graph)?;
    let valid_colors = get_universal_colors(&graph, &assertion_result);
    println!("Assertions evaluated.");

    if valid_colors.is_empty() {
        println!("No color satisfies given assertions. Aborting.");
        return write_empty_report(&assertions, output_zip).map_err(|e| format!("{e:?}"));
    }

    // restrict the colors on the symbolic graph
    let graph = SymbolicAsyncGraph::with_custom_context(
        bn,
        graph.symbolic_context().clone(),
        valid_colors.as_bdd().clone(),
    )?;

    println!("Evaluating properties...");
    // Model check all properties on the restricted graph.
    let property_result = model_check_trees(property_trees, &graph)?;
    let property_colors: Vec<GraphColors> = property_result
        .iter()
        .map(|result| get_universal_colors(&graph, result))
        .collect();

    // do the classification while printing the report and dumping resulting BDDs
    println!("Classifying based on model-checking results...");
    write_class_report_and_dump_bdds(
        &assertions,
        valid_colors,
        &named_properties,
        &property_colors,
        output_zip,
        num_hctl_vars,
        aeon_str.as_str(),
    )
    .map_err(|e| format!("{e:?}"))?;
    println!("Results saved to {}.", output_zip);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::classification::{
        build_combined_assertion, extract_properties, read_model_assertions, read_model_properties,
    };
    use biodivine_hctl_model_checker::model_checking::collect_unique_hctl_vars;
    use biodivine_hctl_model_checker::preprocessing::parser::parse_and_minimize_hctl_formula;
    use biodivine_lib_param_bn::{BooleanNetwork, ModelAnnotation};
    use std::cmp::max;
    use std::collections::HashSet;

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

        let mut var_count = 0;
        for f in formulae {
            let tree = parse_and_minimize_hctl_formula(&bn, f.as_str()).unwrap();
            let c = collect_unique_hctl_vars(tree, HashSet::new()).len();
            var_count = max(c, var_count);
        }
        assert_eq!(var_count, 2);
    }

    #[test]
    /// Test combining of assertion formulae into one conjunction formula.
    fn test_assertion_formulae_merge() {
        let formula1 = "3{x}: @{x}: AX {x}".to_string();
        let formula2 = "false".to_string();
        let formula3 = "a & b".to_string();

        // empty vector should result in true constant
        assert_eq!(build_combined_assertion(&[]), "true".to_string());

        // otherwise, result should be a conjunction ending with `& true`
        assert_eq!(
            build_combined_assertion(&[formula1.clone(), formula2.clone()]),
            "(3{x}: @{x}: AX {x}) & (false)".to_string(),
        );
        assert_eq!(
            build_combined_assertion(&[formula1, formula2, formula3]),
            "(3{x}: @{x}: AX {x}) & (false) & (a & b)".to_string(),
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

        let annotations = ModelAnnotation::from_model_string(aeon_str);
        let assertions = read_model_assertions(&annotations);
        let named_properties = read_model_properties(&annotations).unwrap();

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
        let annotations = ModelAnnotation::from_model_string(aeon_str);
        let props = read_model_properties(&annotations);
        assert!(props.is_err());
        assert_eq!(
            props.err().unwrap().as_str(),
            "Found multiple properties named `p1`."
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
            extract_properties(&named_props),
            vec!["true".to_string(), "false".to_string()]
        );
    }
}
