#!/bin/bash

PARALLEL_INSTANCES=1

# benchmarking model checking for all 4 formulae on 230 fully specified models
python3 run.py 1h models/fully-specified-models model-checking-p1.py -p $PARALLEL_INSTANCES
python3 run.py 1h models/fully-specified-models model-checking-p2.py -p $PARALLEL_INSTANCES
python3 run.py 1h models/fully-specified-models model-checking-p3.py -p $PARALLEL_INSTANCES
python3 run.py 1h models/fully-specified-models model-checking-p4.py -p $PARALLEL_INSTANCES

# benchmarking classification on 230 fully specified models
python3 run.py 1h models/fully-specified-models classification.py -p $PARALLEL_INSTANCES

# parameter scan for 1024 instances of each of the 7 models
python3 run.py 1h models/parameter-scan/018 model-checking-p2.py -p $PARALLEL_INSTANCES
python3 run.py 1h models/parameter-scan/019 model-checking-p2.py -p $PARALLEL_INSTANCES
python3 run.py 1h models/parameter-scan/056 model-checking-p2.py -p $PARALLEL_INSTANCES
python3 run.py 1h models/parameter-scan/077 model-checking-p2.py -p $PARALLEL_INSTANCES
python3 run.py 1h models/parameter-scan/132 model-checking-p2.py -p $PARALLEL_INSTANCES
python3 run.py 1h models/parameter-scan/137 model-checking-p2.py -p $PARALLEL_INSTANCES
python3 run.py 1h models/parameter-scan/207 model-checking-p2.py -p $PARALLEL_INSTANCES

# coloured model checking for all 7 PSBNs
python3 run.py 1h models/parameter-scan/PSBNs model-checking-p2.py

# classification for all 5 PSBNs with higher-arity function symbols
python3 run.py 1h models/large-parametrized-models classification.py
