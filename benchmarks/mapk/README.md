Model of MAPK signalling pathway used in the evaluation.

The folder contains:
- Annotated model (input for the classifier) `model-with-properties.aeon`.
- List containing only the HCTL formulae `properties-only.txt`.
- Pure model without annotations `model-only.aeon`.
- Expected classification results `results-bundle.zip`.
- Expected automatically generated decision tree `decision-tree.png`.

To reproduce the `results-bundle.zip`, run:

```bash
bn-classifier --output-zip results-bundle.zip benchmarks/mapk/model-with-properties.aeon
```

The expected output of the classifier binary is:

```
Loading input files...
Loaded all inputs.
Parsing formulae and generating model representation...
Successfully parsed 1 assertions and 4 properties.
Successfully generated model with 18 vars and 4 params.
Evaluating assertions...
Assertions evaluated.
Evaluating properties...
Classifying based on model-checking results...
Results saved to results-bundle.zip.
Total computation time: 127ms
```

To visualize the decision tree for this model, run:

```bash
hctl-explorer results-bundle.zip
```

In the explorer window, select the root node with *Mixed outcomes* (the only node), drag the *Depth* slider
next to *Auto Expand* to its maximum value, and then click *Auto Expand*. The window
should now show a tree that is equivalent to the one in `decision-tree.png` (up to
the node layout, which is not necessarily deterministic).

Model information:
- Publication: http://dx.doi.org/10.1371/journal.pcbi.1003286
- Model sources:
    - [biodivine-boolean-models](https://github.com/sybila/biodivine-boolean-models/tree/main/models/%5Bid-090%5D__%5Bvar-14%5D__%5Bin-4%5D__%5BMAPK-REDUCED-2%5D)
    - [GINsim](http://ginsim.org/node/173)
