#!/bin/bash

# benchmarking model checking for all 4 formulae on a subset of 100 fully specified models
python3 run.py 1h models/fully-specified-models-subset model-checking-p1.py
python3 run.py 1h models/fully-specified-models-subset model-checking-p2.py
python3 run.py 1h models/fully-specified-models-subset model-checking-p3.py
python3 run.py 1h models/fully-specified-models-subset model-checking-p4.py

# benchmarking classification on subset of 100 fully specified models
python3 run.py 1h models/fully-specified-models-subset classification.py

# parameter scan for 1024 instances of model 077
python3 run.py 1h models/parameter-scan/077 model-checking-p2.py

# coloured model checking for all 5 PSBNs
python3 run.py 1h models/parameter-scan/PSBNs-subset model-checking-p2.py

# classification for all 5 PSBNs with higher-arity function symbols
python3 run.py 1h models/large-parametrized-models classification.py
