use crate::bdt::_impl_bdt_node::class_list_cardinality;
use crate::bdt::{AttributeId, Bdt, BdtNode, BdtNodeId, Outcome, OutcomeMap};
use crate::util::Functional;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColors;
use json::JsonValue;
use std::collections::HashSet;

impl BdtNode {
    /// Convert this BDT node to json value with all available information stored in the node.
    ///
    /// The conversion accepts a precision attribute (see `Bdt` documentation) which can turn
    /// a mixed/decision node into a leaf node during conversion.
    pub fn to_json(&self, precision: Option<u32>) -> JsonValue {
        match self {
            BdtNode::Leaf { class, params } => object! {
                "type" => "leaf".to_string(),
                "cardinality" => params.approx_cardinality(),
                "class" => format!("{class}"),
            },
            BdtNode::Unprocessed { classes } => {
                if let Some((major_class, major_params)) = get_majority_class(classes, precision) {
                    object! {
                        "type" => "leaf".to_string(),
                        "cardinality" => major_params.approx_cardinality(),
                        "class" => major_class.to_string(),
                        "all_classes" => class_list_to_json(classes),
                    }
                } else {
                    object! {
                        "type" => "unprocessed".to_string(),
                        "cardinality" => class_list_cardinality(classes),
                        "classes" => class_list_to_json(classes),
                    }
                }
            }
            BdtNode::Decision {
                attribute,
                left,
                right,
                classes,
            } => {
                if let Some((major_class, major_params)) = get_majority_class(classes, precision) {
                    object! {
                        "type" => "leaf".to_string(),
                        "cardinality" => major_params.approx_cardinality(),
                        "class" => major_class.to_string(),
                        "all_classes" => class_list_to_json(classes),
                    }
                } else {
                    object! {
                        "type" => "decision".to_string(),
                        "cardinality" => class_list_cardinality(classes),
                        "classes" => class_list_to_json(classes),
                        "attribute_id" => attribute.0,
                        "left" => left.0,
                        "right" => right.0,
                    }
                }
            }
        }
    }
}

impl Bdt {
    /// Convert the whole tree into one json array.
    pub fn to_json(&self) -> JsonValue {
        let ids = self
            .storage
            .keys()
            .map(|id| BdtNodeId(*id))
            .collect::<HashSet<_>>();
        self.to_json_partial(&ids)
    }

    /// A variant of `to_json` which allows to specify a subset of IDs that are considered during
    /// export. Other nodes are not included in the result.
    pub fn to_json_partial(&self, ids: &HashSet<BdtNodeId>) -> JsonValue {
        // The order of nodes is irrelevant, but we only want to include nodes that are relevant
        // with the selected precision.
        let mut json_array = JsonValue::Array(vec![]);
        let Some(root) = self.root_id() else {
            return json_array;
        };
        let mut stack = vec![root];
        while let Some(top) = stack.pop() {
            let node_json = self.node_to_json(top);
            if node_json.has_key("left") {
                let left = node_json["left"].as_usize().unwrap();
                let left_id = BdtNodeId::try_from(left, self).unwrap();
                stack.push(left_id);
            }
            if node_json.has_key("right") {
                let right = node_json["right"].as_usize().unwrap();
                let right_id = BdtNodeId::try_from(right, self).unwrap();
                stack.push(right_id);
            }
            if ids.contains(&top) {
                json_array.push(node_json).unwrap();
            }
        }
        json_array
    }

    /// Convert a BDT node to json, including extra info compared to `BDTNode::to_json`.
    ///
    /// The extra info covers the node id as well as attribute name for decision nodes.
    pub fn node_to_json(&self, id: BdtNodeId) -> JsonValue {
        self[id].to_json(self.precision).apply(|result| {
            // Node conversion has no idea about node ids or attribute names, so we need to add
            // them after the fact.
            result.insert("id", id.0).unwrap();
            if result.has_key("attribute_id") {
                let attr_id: AttributeId = result["attribute_id"]
                    .as_usize()
                    .and_then(|i| AttributeId::try_from(i, self))
                    .unwrap();
                result
                    .insert("attribute_name", self[attr_id].name.clone())
                    .unwrap();
            }
        })
    }

    /// Compute attribute gains for the given tree node.
    pub fn attribute_gains_json(&self, id: BdtNodeId) -> JsonValue {
        self.applied_attributes(id)
            .into_iter()
            .map(|it| {
                object! {
                    "id" => it.attribute.to_index(),
                    "name" => self[it.attribute].name.clone(),
                    "left" => class_list_to_json(&it.left),
                    "right" => class_list_to_json(&it.right),
                    "gain" => it.information_gain
                }
            })
            .collect::<Vec<_>>()
            .and_then(JsonValue::from)
    }
}

pub(super) fn class_list_to_json(classes: &OutcomeMap) -> JsonValue {
    JsonValue::from(classes.iter().map(class_to_json).collect::<Vec<_>>())
}

pub(super) fn class_to_json((class, params): (&Outcome, &GraphColors)) -> JsonValue {
    object! {
        "cardinality" => params.approx_cardinality(),
        "class" => format!("{class}"),
    }
}

pub(super) fn get_majority_class(
    classes: &OutcomeMap,
    precision: Option<u32>,
) -> Option<(&Outcome, &GraphColors)> {
    if let Some(precision) = precision {
        let total_node_cardinality = class_list_cardinality(classes);
        let return_when = total_node_cardinality * (f64::from(precision) / 10000.0);
        for (class, params) in classes {
            if params.approx_cardinality() >= return_when {
                return Some((class, params));
            }
        }
        None
    } else {
        None
    }
}
