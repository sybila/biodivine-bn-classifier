Model of T-LGL survival network.

The folder contains:
- Annotated model (input for the classifier) `model-with-properties.aeon`.
- List containing only the HCTL formulae `properties-only.txt`.
- Pure model without annotations `model-only.aeon`.
- Expected classification results `results-bundle.zip`.
- Expected automatically generated decision tree `decision-tree.png`.

To reproduce the `results-bundle.zip`, run:

```bash
bn-classifier --output-zip results-bundle.zip benchmarks/tlgl/model-with-properties.aeon
```

The expected output of the classifier binary is:

```
Loading input files...
Loaded all inputs.
Parsing formulae and generating model representation...
Successfully parsed 3 assertions and 4 properties.
Successfully generated model with 61 vars and 7 params.
Evaluating assertions...
Assertions evaluated.
Evaluating properties...
Classifying based on model-checking results...
Results saved to results-bundle.zip.
Total computation time: 55731ms
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
- Publication: https://doi.org/10.1073/pnas.0806447105
- Model sources: 
  - [biodivine-boolean-models](https://github.com/sybila/biodivine-boolean-models/tree/main/models/%5Bid-014%5D__%5Bvar-54%5D__%5Bin-7%5D__%5BT-LGL-SURVIVAL-NETWORK-2008%5D)
  - [Cell Collective](https://research.cellcollective.org/?dashboard=true#module/2176:1/tlgl-survival-network-2008/1)
  - [GINsim](http://ginsim.org/node/87)
