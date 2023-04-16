## Scalability evaluation

To evaluate the scalability of the classification procedure, we tested
our tool on a collection of several complex properties and large parametrized biological models.

All experiments were performed on a standard laptop with an 8-th Gen Intel i7 CPU and 8GB RAM.

### Properties

We have selected 4 general properties to run a classification on each model (always the same properties). 
These properties describe various types of long-term behaviour, which is important for studying biological systems.
Particularly, the first two are concerned with existence of different variants of periodically visited states, 
and the other two with multi-stability of the system.
```
# 1) existence of a "checkpoint" state (state that does not have a self-loop
#    and all its outgoing paths inevitably lead to the state itself)
#! dynamic_property: p1: #`3{x}: @{x}: (AX (~{x} & AF {x}))`#

# 2) existence of a "checkpoint" state that lies on at least two different cycles
#! dynamic_property: p2: #`3{x}: @{x}: ((AX (~{x} & AF {x})) & (EF (!{y}: EX ~AF {y})))`#

# 3) existence of a pair of steady states
#! dynamic_property: p3: #`3{x}: 3{y}: ((@{x}: ~{y} & AX {x}) & (@{y}: AX {y}))`#

# 4) existence of a "fork" state in the system (state where system decides which
#    terminal steady-state to visit)
#! dynamic_property: p4: #`3{x}: 3{y}: ((@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z}))))`#
```

### Models

Each subdirectory of this folder is dedicated to one tested model.
These sub-directories contain following:
- model in aeon format annotated with the properties (input for the classifier) `model-parametrized-annotated.aeon`
- archive with classification results `model-results.zip` 
- for some models, we also include non-parametrized and non-annotated version of the model `model-fixed.aeon`
(this may be useful for converting the model to formats that do not support parameters or properties)
