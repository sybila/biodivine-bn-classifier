//! Loading of various input components of the model, mainly of various properties/assertions.

use biodivine_lib_param_bn::ModelAnnotation;

type NamedFormulaeVec = Vec<(String, String)>;

/// Read the list of assertions from an `.aeon` model annotation object.
///
/// The assertions are expected to appear as `#!dynamic_assertion: FORMULA` model annotations
/// and they are returned in declaration order.
pub fn read_model_assertions(annotations: &ModelAnnotation) -> Vec<String> {
    let Some(list) = annotations.get_value(&["dynamic_assertion"]) else {
        return Vec::new();
    };
    list.lines().map(|it| it.to_string()).collect()
}

/// Read the list of named properties from an `.aeon` model annotation object.
///
/// The properties are expected to appear as `#!dynamic_property: NAME: FORMULA` model annotations.
/// They are returned in alphabetic order w.r.t. the property name.
pub fn read_model_properties(annotations: &ModelAnnotation) -> Result<NamedFormulaeVec, String> {
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
pub fn build_combined_assertion(assertions: &[String]) -> String {
    if assertions.is_empty() {
        "true".to_string()
    } else {
        // Add parenthesis to each assertion.
        let assertions: Vec<String> = assertions.iter().map(|it| format!("({it})")).collect();
        // Join them into one big conjunction.
        assertions.join(" & ")
    }
}

/// Extract properties from name-property pairs.
pub fn extract_properties(named_props: &NamedFormulaeVec) -> Vec<String> {
    named_props.iter().map(|(_, x)| x.clone()).collect()
}
