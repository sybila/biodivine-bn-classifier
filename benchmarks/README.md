# BN Classifier Benchmarks

This repository is configured to allow painless running of non-trivial benchmarks of the tools `classification` engine and its `model-checking` component (as described in Section 6.2 and 6.3 of the paper). If you want to replicate the results of the case study, follow the provided Tutorial.

This README contains the following (in this order):
- description of provided sets of benchmark models
- description of provided benchmarking scripts
- description of provided precomputed results
- instructions on how to execute the large prepared benchmark sets
- instructions on how to run (quick) experiments on selected models

### Benchmark models

All benchmark models are present in the `models` directory. They are all originally taken from [BBM repository](https://github.com/sybila/biodivine-boolean-models).

- `fully-specified-models` folder contains 230 fully specified models that we used to evaluate the tool's performance in Section 6.2 of the paper
- `parameter-scan` folder contains all models related to our parameter scan experiments in Section 6.3 of the paper
    - `PSBNs` sub-folder contains the seven selected PSBNs (that we run the coloured model checking on)
    - other seven sub-folders each contain the 1024 sampled instances for one of the selected PSBN models (that we use to estimate the parameter scan time)
- `large-parametrized-models` folder contains PSBN models involving function symbols of higher arity (that we mention in the end of Section 6.3 of the paper)


### Benchmark scripts 

The `run.py` script ("runner") makes it possible to run a particular Python script for each model ("experiment") in a directory (e.g. `fully-specified-models`). The runner then ensures that each experiment runs within a specified timeout, that the runtime is measured, and that the output is saved to a separate file.

The runner can operate in three modes:

1. Interactive: Before each experiment is started, you are prompted to either run it or skip it. You can also abort the whole run.
2. Sequential: Experiments run automatically one after the other.
3. Parallel: Up to `N` (user-provided) experiments are executed in parallel (as separate processes).

Executing the runner script:
```
python3 run.py TIMEOUT BENCH_DIR SCRIPT [-i/-p N]
```

 - `TIMEOUT` is a standard UNIX timeout string. E.g. `1h`, `10s`, etc.
 - `BENCH_DIR` is a path to a directory that will be scanned to obtain the list of experiments (`.aeon` files).
 - `SCRIPT` is a path to a Python script that will be started by the runner, with an experiment file (`.aeon` file) as the first argument. Naturally, this script can then start other native processes if necessary, or modify the model as desired.
 - Add `-i` if you want to use the interactive mode.
 - Add `-p N` if you want to run up to `N` experiments in parallel.

> WARNING: The script does not respond particularly well to keyboard interrupts. If you kill the benchmark runner (e.g. `Ctrl+C`), you may also need to terminate some unfinished experiments manually.

> ANOTHER WARNING: For some reason, not all exit codes are always propagated correctly through the whole `python <-> timeout <-> time <-> python` chain. For this reason, benchmarks that run out of memory can still result in a "successful" measurement (successful in the sense that it finished before timeout). For this reason, always run `helper-scripts/postprocess-results-mc.py` or `helper-scripts/postprocess-results-classif.py` (depending on what benchmark script you ran) on the directory with results.

We provide the following scripts to execute via the runner
- `model-checking-p1.py` to evaluate formula p1 (phi1) on a model
- `model-checking-p2.py` to evaluate formula p2 (phi2) on a model
- `model-checking-p3.py` to evaluate formula p3 (phi3) on a model
- `model-checking-p4.py` to evaluate formula p4 (phi4) on a model
- `classification.py` to run the whole classification process on an annotated model

### Precomputed results

All the results of our experiments are present in the `results` directory.

- `classification` folder contains results of classification on models from `models/fully-specified-models` (as discussed in Section 6.2 of the paper)
- `model-checking` folder contains the model-checking results on models from `models/fully-specified-models`, one sub-folder for each of the four formulae (also discussed in Section 6.2)
- `coloured-model-checking` contains results of the coloured model checking on seven selected PSBNs from `models/parameter-scan/PSBNs` (as discussed in Section 6.3 of the paper)
- `parameter-scan` contains results of the parameter scan for all 7 selected models in `models/parameter-scan/*` (as discussed in Section 6.3 of the paper)
- `large-parametrized-models` contains results of classification on models from `models/large-parametrized-models` (as mentioned in the end of Section 6.3 of the paper)

Classification results for each model consists of `*_out.txt` with command line output (progress summary, ...), and `*-archive.zip` with the resulting classification archive.
Model-checking results for each model consist of `*_out.txt` command line output (summary of how many state-colour pairs, states, colours satisfy the property, ...). 

If an experiment successfully finished in one hour, the corresponding `*_out.txt` contains (besides other things) the computation time. If the process does not finish due to timeout or out-of-memory, it is also mentioned so in the `*_out.txt` file. For results of the large benchmark sets, there also two `.csv` files with aggregated results, as produced by the runner script.


### Executing prepared benchmarks

There are several pre-configured benchmark commands that you can use. Of course, you can add `-p N` or `-i` to each command involving the runner script to run it in parallel or interactive mode.

Note that executing the full performance benchmarks and parameter scan benchmarks might take a long time (up to several hours with added option `-p 16` for 16 parallel processes), because it involves hundreds (thousands) of models.

> Always make sure that the provided Python virtual environment is active before executing the experiments.

To run the full prepared performance benchmarks:
```
# model checking for each formula
python3 run.py 1h models/fully-specified-models model-checking-p1.py
python3 run.py 1h models/fully-specified-models model-checking-p2.py
python3 run.py 1h models/fully-specified-models model-checking-p3.py
python3 run.py 1h models/fully-specified-models model-checking-p4.py

# classification
python3 run.py 1h models/fully-specified-models classification.py
```

To run the full prepared parameter scan benchmarks, choose model `ID`, substitute it in the command, and run:
```
# parameter scan for 1024 selected instances of a model with given `ID`
python3 run.py 1h models/parameter-scan/ID model-checking-p2.py

# coloured model checking for all 7 PSBNs
python3 run.py 1h models/parameter-scan/PSBNs model-checking-p2.py

# coloured model checking for a single PSBN model with given `ID`
python3 model-checking-p2.py models/parameter-scan/PSBNs/ID.aeon
```

To run the prepared benchmarks for PSBN models with higher-arity function symbols:
```
# classification for all 5 PSBNs
python3 run.py 1h models/large-parametrized-models classification.py

# classification for a single selected `MODEL`
python3 classification.py models/large-parametrized-models/MODEL
```

### Running own experiments on selected models

You can also directly execute scripts `classification.py` or `model-checking.py` on selected models to run single experiments. To construct own formulae, choose from the ones provided in plaintext in `plain-text-formulae.txt` or see the tool's Manual.

```
python3 classification.py MODEL_PATH
python3 model-checking.py MODEL_PATH --formula HCTL_FORMULA
```

In particular, the following experiments should finish in under 1 minute on a standard laptop and should not require too much memory:

1) model checking and classification experiments on fully specified model `206` with 41 variables
```
python3 classification.py models/fully-specified-models/206.aeon

python3 model-checking.py models/fully-specified-models/206.aeon --formula '3{x}: (@{x}: (AX (~{x} & AF {x})))'
```

2) classification experiment on a PSBN with 66 variables and 2^72 instances
```
python3 classification.py models/large-parametrized-models/[Butanol-production-signalling]-[66v]-[72p].aeon
```

3) model-checking experiment on a PSBN with 28 variables and 2^72 instances
```
python3 model-checking.py models/large-parametrized-models/[FA-BRCA-pathway]-[28v]-[72p].aeon --formula '3{x}: 3{y}: ((@{x}: ~{y} & AX {x}) & (@{y}: AX {y}))'
```