use crate::bdt::{BdtNode, OutcomeMap};
use crate::Outcome;
use std::collections::HashSet;

impl BdtNode {
    /// Computes the cardinality of the parameter set covered by this tree node.
    pub fn approx_cardinality(&self) -> f64 {
        match self {
            BdtNode::Leaf { params, .. } => params.approx_cardinality(),
            BdtNode::Decision { classes, .. } => class_list_cardinality(classes),
            BdtNode::Unprocessed { classes, .. } => class_list_cardinality(classes),
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, BdtNode::Leaf { .. })
    }

    pub fn is_decision(&self) -> bool {
        matches!(self, BdtNode::Decision { .. })
    }

    pub fn is_unprocessed(&self) -> bool {
        matches!(self, BdtNode::Unprocessed { .. })
    }

    /// Computes properties that are universally satisfied in that node.
    /// It is hacked a bit for now - assumes that class (outcome) names are of following
    /// format: "sat_property_1, sat_property_2, ..., sat_property_n (COUNT)" or "- (COUNT)"
    pub fn universally_satisfied_props(&self) -> HashSet<&str> {
        match self {
            BdtNode::Leaf { class, .. } => parse_properties_from_class(class),
            BdtNode::Decision { classes, .. } | BdtNode::Unprocessed { classes, .. } => {
                // Start with any class and then compute intersection of all remaining classes.
                let mut keys = classes.keys();
                let mut sat_props = keys
                    .next()
                    .map(parse_properties_from_class)
                    .unwrap_or_else(HashSet::new);
                for class in keys {
                    let class_props = parse_properties_from_class(class);
                    sat_props.retain(|item| class_props.contains(item));
                }
                sat_props
            }
        }
    }

    /// Computes properties that are satisfied in at least one outcome covered by this node.
    ///
    /// Same caveats as for `universally_satisfied_props` apply.
    pub fn existentially_satisfied_props(&self) -> HashSet<&str> {
        match self {
            BdtNode::Leaf { class, .. } => parse_properties_from_class(class),
            BdtNode::Decision { classes, .. } | BdtNode::Unprocessed { classes, .. } => {
                // Start with any class and then compute union of all remaining classes.
                let mut sat_props = HashSet::new();
                for class in classes.keys() {
                    sat_props.extend(parse_properties_from_class(class));
                }
                sat_props
            }
        }
    }
}

/// **(internal)** Parses a class (outcome) name (of format: "prop1, sat_prop2, ..., prop_n" into
/// individual properties which define that class.
fn parse_properties_from_class(outcome: &Outcome) -> HashSet<&str> {
    let outcome_name = outcome.0.as_str();
    // if no property is satisfied, name of the class is "- (COUNT)"
    if outcome_name == "-" || outcome_name.starts_with("- ") {
        return HashSet::new();
    }
    let prefix = outcome_name.split('(').next().unwrap();
    prefix.split(',').map(|it| it.trim()).collect()
}

/// **(internal)** Utility method for computing cardinality of a collection of classes.
pub(super) fn class_list_cardinality(classes: &OutcomeMap) -> f64 {
    classes
        .iter()
        .fold(0.0, |a, (_, b)| a + b.approx_cardinality())
}
