## Scalability evaluation

To evaluate the scalability of the classification procedure, we tested
`bn-classifier` on a collection of several complex properties and large parametrized biological models.

All experiments were performed on a standard laptop with an 8-th Gen Intel i7 CPU and 8GB RAM.

### Properties

We have selected 4 general properties to run a classification on each model (always the same properties). 
These properties describe various types of long-term behaviour, which is important for studying biological systems.
In particular, the first two are concerned with existence of different variants of periodically visited states, 
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
These subdirectories contain the following:
- Model in aeon format annotated with the properties (input for the classifier): `model-parametrized-annotated.aeon`
- Archive with expected classification results: `model-results.zip` 
- For some models, we also include non-parametrized and non-annotated version of the model `model-fixed.aeon`
(this may be useful for converting the model to formats that do not support parameters or property annotations).

### Benchmarking

To run each benchmark, simply execute:

```bash
bn-classifier --output-dir result.zip ./{MODEL_DIR}/{MODEL_NAME}-parametrized-annotated.aeon
```

The standard output should contain the running time for the whole computation.

### Results

The following table gives a summary of the expected results. Naturally, the actual runtime
will differ based on the performance of your machine.

|   Model   | State space | Parametrisations |         Runtime |
|:---------:|:-----------:|:----------------:|----------------:|
| Apoptosis |   2.19e12   |      8.61e8      |         39350ms |
|  Butanol  |   7.37e19   |      8.18e9      |         18310ms |
|   EGFR    |   2.02e31   |        9         |        209698ms |
|   MAPK    |   2.62e5    |     1.12e15      |          4949ms |
|  FA BRCA  |   2.68e8    |      1.52e9      |        153120ms |
