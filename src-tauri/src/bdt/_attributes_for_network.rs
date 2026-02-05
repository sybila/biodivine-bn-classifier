use crate::bdt::{Attribute, AttributeContext, Bdt, OutcomeMap};
use crate::util::Functional;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::{FnUpdate, VariableId};
use std::collections::HashMap;

impl Bdt {
    pub fn new_from_graph(
        classes: OutcomeMap,
        graph: &SymbolicAsyncGraph,
        named_properties: HashMap<String, String>,
        attribute_combinations: &[String],
    ) -> Bdt {
        // TODO:
        //  This process is somewhat weird if the graph has actual "parameters",
        //  because the unit set of the model can eliminate some combinations of constants
        //  based on monotonicity rules.
        let mut attributes = Vec::new();
        attributes_for_network_inputs(graph, &mut attributes);
        attributes_for_constant_parameters(graph, &mut attributes);
        attributes_for_missing_constraints(graph, &mut attributes);
        attributes_for_implicit_function_tables(graph, &mut attributes);
        attributes_for_explicit_function_tables(graph, &mut attributes);
        attributes_for_conditional_observability(graph, &mut attributes);
        for combination in attribute_combinations {
            let attribute = read_attribute_combination(combination, &attributes).unwrap();
            attributes.push(attribute);
        }
        println!("Generated attributes: {}", attributes.len());
        let attributes: Vec<Attribute> = attributes
            .into_iter()
            .filter(|a| {
                // For now, this part is disabled due to some weird unit set behavior in
                // models with zero-arity parameters.
                /*let is_not_empty =
                    is_not_empty && !a.positive.intersect(graph.unit_colors()).is_empty();
                let is_not_empty =
                    is_not_empty && !a.negative.intersect(graph.unit_colors()).is_empty();*/
                !a.positive.is_empty() && !a.negative.is_empty()
            })
            .collect();
        println!("Filtered attributes: {}", attributes.len());
        Bdt::new(classes, attributes, named_properties)
    }
}

pub fn mk_conjunction_attribute(a1: &Attribute, a2: &Attribute) -> Attribute {
    assert!(a1.context.is_none() && a2.context.is_none());
    Attribute {
        name: format!("{} & {}", a1.name, a2.name),
        positive: a1.positive.intersect(&a2.positive),
        negative: a1.negative.union(&a2.negative),
        context: None,
    }
}

pub fn mk_disjunction_attribute(a1: &Attribute, a2: &Attribute) -> Attribute {
    assert!(a1.context.is_none() && a2.context.is_none());
    Attribute {
        name: format!("{} | {}", a1.name, a2.name),
        positive: a1.positive.union(&a2.positive),
        negative: a1.negative.intersect(&a2.negative),
        context: None,
    }
}

pub fn read_attribute_combination(
    attribute_combination: &str,
    existing: &[Attribute],
) -> Result<Attribute, String> {
    if attribute_combination.contains("&") && !attribute_combination.contains("|") {
        let mut conjunction: Option<Attribute> = None;
        for x in attribute_combination.split("&") {
            let attr = existing
                .iter()
                .find(|a| a.name == x.trim())
                .ok_or_else(|| format!("Cannot find attribute `{}`", x))?;
            if let Some(other) = conjunction {
                conjunction = Some(mk_conjunction_attribute(&other, attr));
            } else {
                conjunction = Some(attr.clone());
            }
        }
        conjunction
            .ok_or_else(|| format!("Empty attribute combination: `{}`", attribute_combination))
    } else if attribute_combination.contains("|") && !attribute_combination.contains("&") {
        let mut disjunction: Option<Attribute> = None;
        for x in attribute_combination.split("|") {
            let attr = existing
                .iter()
                .find(|a| a.name == x.trim())
                .ok_or_else(|| format!("Cannot find attribute `{}`", x))?;
            if let Some(other) = disjunction {
                disjunction = Some(mk_disjunction_attribute(&other, attr));
            } else {
                disjunction = Some(attr.clone());
            }
        }
        disjunction
            .ok_or_else(|| format!("Empty attribute combination: `{}`", attribute_combination))
    } else {
        Err(format!("Attribute combination `{attribute_combination}` must be either a conjunction (&) or disjunction (|)."))
    }
}

/// **(internal)** Construct basic attributes for all input variables.
fn attributes_for_network_inputs(graph: &SymbolicAsyncGraph, out: &mut Vec<Attribute>) {
    let network = graph.as_network().unwrap();
    for v in network.variables() {
        // v is input if it has no update function and no regulators
        let is_input = network.regulators(v).is_empty();
        let is_input = is_input && network.get_update_function(v).is_none();
        if is_input {
            let bdd = graph
                .symbolic_context()
                .mk_implicit_function_is_true(v, &[]);
            out.push(Attribute {
                name: network.get_variable_name(v).clone(),
                negative: graph.empty_colors().copy(bdd.not()),
                positive: graph.empty_colors().copy(bdd),
                context: None,
            })
        }
    }
}

/// **(internal)** Construct basic attributes for all constant parameters of the network.
fn attributes_for_constant_parameters(graph: &SymbolicAsyncGraph, out: &mut Vec<Attribute>) {
    let network = graph.as_network().unwrap();
    for p in network.parameters() {
        println!("{:?}: {}", p, network[p].get_name());
        if network[p].get_arity() == 0 {
            // Parameter is a constant
            let bdd = graph
                .symbolic_context()
                .mk_uninterpreted_function_is_true(p, &[]);
            out.push(Attribute {
                name: network[p].get_name().clone(),
                negative: graph.empty_colors().copy(bdd.not()),
                positive: graph.empty_colors().copy(bdd),
                context: None,
            })
        }
    }
}

/// **(internal)** If some regulation has a missing static constraint, try to build it
/// and add it as an attribute.
fn attributes_for_missing_constraints(graph: &SymbolicAsyncGraph, out: &mut Vec<Attribute>) {
    let network = graph.as_network().unwrap();
    let context = graph.symbolic_context();
    for reg in network.as_graph().regulations() {
        // This is straight up copied from static constraint analysis in lib-param-bn.
        // For more context, go there.
        let target = reg.get_target();
        let update_function = network.get_update_function(target);
        let fn_is_true = if let Some(function) = update_function {
            context.mk_fn_update_true(function)
        } else {
            context.mk_implicit_function_is_true(target, &network.regulators(target))
        };
        let fn_is_false = fn_is_true.not();
        let regulator: usize = reg.get_regulator().into();
        let regulator = context.state_variables()[regulator];
        let regulator_is_true = context.mk_state_variable_is_true(reg.get_regulator());
        let regulator_is_false = context.mk_state_variable_is_true(reg.get_regulator()).not();

        if !reg.is_observable() {
            let fn_x1_to_1 = fn_is_true.and(&regulator_is_true).var_exists(regulator);
            let fn_x0_to_1 = fn_is_true.and(&regulator_is_false).var_exists(regulator);
            let observability = fn_x1_to_1
                .xor(&fn_x0_to_1)
                .exists(context.state_variables());

            out.push(Attribute {
                name: format!(
                    "{} essential in {}",
                    network.get_variable_name(reg.get_regulator()),
                    network.get_variable_name(reg.get_target()),
                ),
                negative: graph.empty_colors().copy(observability.not()),
                positive: graph.empty_colors().copy(observability),
                context: Some(AttributeContext {
                    regulator: reg.get_regulator(),
                    target: reg.get_target(),
                    context: vec![],
                }),
            });
        }

        if reg.get_monotonicity().is_none() {
            let fn_x1_to_0 = fn_is_false.and(&regulator_is_true).var_exists(regulator);
            let fn_x0_to_1 = fn_is_true.and(&regulator_is_false).var_exists(regulator);
            let non_activation = fn_x0_to_1
                .and(&fn_x1_to_0)
                .exists(context.state_variables());

            let fn_x0_to_0 = fn_is_false.and(&regulator_is_false).var_exists(regulator);
            let fn_x1_to_1 = fn_is_true.and(&regulator_is_true).var_exists(regulator);
            let non_inhibition = fn_x0_to_0
                .and(&fn_x1_to_1)
                .exists(context.state_variables());

            out.push(Attribute {
                name: format!(
                    "{} activation in {}",
                    network.get_variable_name(reg.get_regulator()),
                    network.get_variable_name(reg.get_target()),
                ),
                positive: graph.empty_colors().copy(non_activation.not()),
                negative: graph.empty_colors().copy(non_activation),
                context: Some(AttributeContext {
                    regulator: reg.get_regulator(),
                    target: reg.get_target(),
                    context: vec![],
                }),
            });

            out.push(Attribute {
                name: format!(
                    "{} inhibition in {}",
                    network.get_variable_name(reg.get_regulator()),
                    network.get_variable_name(reg.get_target()),
                ),
                positive: graph.empty_colors().copy(non_inhibition.not()),
                negative: graph.empty_colors().copy(non_inhibition),
                context: Some(AttributeContext {
                    regulator: reg.get_regulator(),
                    target: reg.get_target(),
                    context: vec![],
                }),
            });
        }
    }
}

/// **(internal)** Make an explicit attributes (like `f[1,0,1] = 1`) for every implicit update
/// function row in the network.
fn attributes_for_implicit_function_tables(graph: &SymbolicAsyncGraph, out: &mut Vec<Attribute>) {
    let network = graph.as_network().unwrap();
    for v in network.variables() {
        let is_implicit_function = network.get_update_function(v).is_none();
        let is_implicit_function = is_implicit_function && !network.regulators(v).is_empty();
        if is_implicit_function {
            let table = graph
                .symbolic_context()
                .get_implicit_function_table(v)
                .unwrap();
            for (ctx, var) in table {
                let bdd = graph.symbolic_context().bdd_variable_set().mk_var(var);
                let ctx: Vec<String> = ctx
                    .into_iter()
                    .zip(network.regulators(v))
                    .map(|(b, r)| {
                        format!(
                            "{}{}",
                            if b { "" } else { "¬" },
                            network.get_variable_name(r)
                        )
                    })
                    .collect();
                let name = format!("{}{:?}", network.get_variable_name(v), ctx);
                out.push(Attribute {
                    name: name.replace('\"', ""),
                    negative: graph.mk_empty_colors().copy(bdd.not()),
                    positive: graph.mk_empty_colors().copy(bdd),
                    context: None,
                });
            }
        }
    }
}

/// **(internal)** Make an explicit argument for every explicit parameter function row in the network.
fn attributes_for_explicit_function_tables(graph: &SymbolicAsyncGraph, out: &mut Vec<Attribute>) {
    let network = graph.as_network().unwrap();
    for p in network.parameters() {
        let parameter = network.get_parameter(p);
        if parameter.get_arity() > 0 {
            let table = graph.symbolic_context().get_explicit_function_table(p);
            let arg_names = (0..parameter.get_arity())
                .map(|i| format!("x{}", i + 1))
                .collect::<Vec<_>>();
            for (ctx, var) in table {
                let bdd = graph.symbolic_context().bdd_variable_set().mk_var(var);
                let ctx: Vec<String> = ctx
                    .into_iter()
                    .zip(&arg_names)
                    .map(|(b, r)| format!("{}{}", if b { "" } else { "¬" }, r))
                    .collect();
                let name = format!("{}{:?}", parameter.get_name(), ctx);
                out.push(Attribute {
                    name: name.replace('\"', ""),
                    negative: graph.mk_empty_colors().copy(bdd.not()),
                    positive: graph.mk_empty_colors().copy(bdd),
                    context: None,
                });
            }
        }
    }
}

/// Create "conditional observability" attributes for both implicit and explicit update functions.
fn attributes_for_conditional_observability(graph: &SymbolicAsyncGraph, out: &mut Vec<Attribute>) {
    let context = graph.symbolic_context();
    let network = graph.as_network().unwrap();
    for v in network.variables() {
        let regulators = network.regulators(v);

        // Bdd that is true when update function for this variable is true
        let fn_is_true = if let Some(function) = network.get_update_function(v) {
            context.mk_fn_update_true(function)
        } else {
            context.mk_implicit_function_is_true(v, &regulators)
        };

        let contexts = if let Some(function) = network.get_update_function(v) {
            variable_contexts(function)
        } else {
            vec![regulators.clone()]
        };

        // Go through all variable combinations for the given context
        for fn_context in contexts {
            // Regulator whose observability are we dealing with
            for r in fn_context.iter().cloned() {
                // Remaining regulators form the "context variables"
                let context_vars: Vec<VariableId> =
                    fn_context.iter().filter(|v| **v != r).cloned().collect();
                // X and !X conditions based on context_vars
                let conditions = context_vars
                    .iter()
                    .flat_map(|c_var| {
                        let bdd_1 = context.mk_state_variable_is_true(*c_var);
                        let bdd_0 = bdd_1.not();
                        let name = network.get_variable_name(*c_var);
                        [(format!("¬{name}"), bdd_0), (name.clone(), bdd_1)].to_vec()
                    })
                    .collect::<Vec<(String, Bdd)>>();
                // All non-empty combinations of conditions
                let contexts = make_contexts(&conditions);

                let r_var: usize = r.into();
                let r_var = context.state_variables()[r_var];
                let regulator_is_true = context.mk_state_variable_is_true(r);
                let regulator_is_false = context.mk_state_variable_is_true(r).not();

                // Unconditional observability is already covered above, so we don't handle it here
                for (condition_name, condition_list, condition_bdd) in contexts {
                    // Restrict to values that satisfy conditions
                    let fn_is_true = fn_is_true.and(&condition_bdd);
                    let fn_x1_to_1 = fn_is_true.and(&regulator_is_true).var_exists(r_var);
                    let fn_x0_to_1 = fn_is_true.and(&regulator_is_false).var_exists(r_var);
                    let observability = fn_x1_to_1
                        .xor(&fn_x0_to_1)
                        .exists(context.state_variables());

                    out.push(Attribute {
                        name: format!(
                            "{} essential in {} for {}",
                            network.get_variable_name(r),
                            network.get_variable_name(v),
                            condition_name,
                        ),
                        negative: graph.empty_colors().copy(observability.not()),
                        positive: graph.empty_colors().copy(observability),
                        context: Some(AttributeContext {
                            target: v,
                            regulator: r,
                            context: condition_list,
                        }),
                    });
                }
            }
        }
    }
}

/// Compute "contexts" of this update function. These are combinations
/// of variables that meet together in one explicit parameter.
fn variable_contexts(function: &FnUpdate) -> Vec<Vec<VariableId>> {
    match function {
        FnUpdate::Const(_) => vec![],
        FnUpdate::Var(_) => vec![],
        FnUpdate::Param(_, args) => {
            let mut arg_vars = Vec::new();
            for arg in args {
                if let FnUpdate::Var(var) = arg {
                    arg_vars.push(*var);
                } else {
                    return vec![];
                }
            }
            vec![arg_vars]
        }
        FnUpdate::Not(inner) => variable_contexts(inner),
        FnUpdate::Binary(_, l, r) => variable_contexts(l)
            .apply(|list| variable_contexts(r).into_iter().for_each(|c| list.push(c))),
    }
}

/// Build all combinations of labelled conditions.
///
/// For example, given X, Y, Z, this will produce:
/// X, Y, Z
/// XY, XZ, YZ,
/// XYZ
///
/// This should also automatically filter out empty results, so you can
/// include A and !A in the conditions without problems.
fn make_contexts(conditions: &[(String, Bdd)]) -> Vec<(String, Vec<String>, Bdd)> {
    fn recursion(
        conditions: &[(String, Bdd)],
        partial_condition: &(String, Vec<String>, Bdd),
        out: &mut Vec<(String, Vec<String>, Bdd)>,
    ) {
        if conditions.is_empty() {
            return;
        }
        for (i, c) in conditions.iter().enumerate() {
            let updated_name = format!("{}, {}", partial_condition.0, c.0);
            let updated_colors = partial_condition.2.and(&c.1);
            if updated_colors.is_false() {
                continue;
            }
            let update_condition_list = partial_condition.1.clone().apply(|i| i.push(c.0.clone()));
            let updated = (updated_name, update_condition_list, updated_colors);
            if i != conditions.len() - 1 {
                recursion(&conditions[(i + 1)..], &updated, out);
            }
            out.push(updated);
        }
    }
    if conditions.is_empty() {
        return vec![];
    }
    let mut result: Vec<(String, Vec<String>, Bdd)> = Vec::new();
    for (i, c) in conditions.iter().enumerate() {
        if c.1.is_false() {
            continue;
        }
        let pair = (c.0.clone(), vec![c.0.clone()], c.1.clone());
        if i != conditions.len() - 1 {
            recursion(&conditions[(i + 1)..], &pair, &mut result);
        }
        result.push(pair);
    }
    result
}
