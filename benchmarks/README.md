# BN Classifier Benchmarks

This repository is configured to allow painless running of non-trivial benchmarks of the tools `classification` engine and its `model-checking` component (as described in Section 6.2 and 6.3 of the paper). If you want to replicate the results of the case study, follow the provided `tutorial` (as instructed in the main readme).

This README contains the following (in this order):
- description of provided sets of benchmark models
- description of provided precomputed results
- description of provided benchmarking scripts
- instructions on how to execute the large prepared benchmark sets
- instructions on how to run (quick) experiments on selected models

## Benchmark models

All benchmark models are present in the `models` directory. They are all originally taken from [BBM repository](https://github.com/sybila/biodivine-boolean-models).

- `fully-specified-models` folder contains all 230 fully specified models that we used to evaluate the tool's performance in Section 6.2 of the paper
- `fully-specified-models-subset` contains a subset of 100 models (out of 230 of the `fully-specified-models`) that we selected to allow for a "swift" evalution (on this subset, all relevant experiments take less than 10 minutes in total - compared to tens of hours when using all 230 models - more on this later)
- `parameter-scan` folder contains all models related to our parameter scan experiments in Section 6.3 of the paper
    - `PSBNs` sub-folder contains the seven selected PSBNs (that we run the coloured model checking on)
    - other seven sub-folders each contain the 1024 sampled instances for one of the selected PSBN models (that we use to estimate the parameter scan time)
- `large-parametrized-models` folder contains PSBN models involving function symbols of higher arity (that we mention in the end of Section 6.3 of the paper)

## Precomputed results

All the results of our experiments are present in the `results` directory.

- `classification` folder contains results of classification on models from `models/fully-specified-models` (as discussed in Section 6.2 of the paper)
- `model-checking` folder contains the model-checking results on models from `models/fully-specified-models`, one sub-folder for each of the four formulae (also discussed in Section 6.2)
- `coloured-model-checking` contains results of the coloured model checking on seven selected PSBNs from `models/parameter-scan/PSBNs` (as discussed in Section 6.3 of the paper)
- `parameter-scan` contains results of the parameter scan for all 7 selected models in `models/parameter-scan/*` (as discussed in Section 6.3 of the paper)
- `large-parametrized-models` contains results of classification on models from `models/large-parametrized-models` (as mentioned in the end of Section 6.3 of the paper)

Classification results for each model consists of `*_out.txt` with command line output (computation time + progress summary) and `*-archive.zip` with the resulting classification archive.
Model-checking results for each model consist of `*_out.txt` command line output (computation time + short summary of model-checking results). 

#### Resources 
Note that we have run all the experiments on on a workstation with a 32-core AMD Threadripper 2990WX CPU and 64GB of RAM. Every experiment was executed in a separate process, and its runtime was measured using the standard UNIX time utility. Due to the high number of considered experiments, we typically executed up to 16 benchmarks in parallel to speed up the evaluation. This is because, in our experience, the memory bandwidth of this particular CPU does not scale sufficiently to all 32 cores for the considered workloads (symbolic algorithms typically perform a lot of sequential random memory accesses, resulting in contention among cores). For each benchmark instance, we consider a timeout of one hour.

If an experiment successfully finished in one hour, the corresponding `*_out.txt` contains (besides other things) the computation time. If the process did not finish due to timeout or out-of-memory, it is also mentioned so in the `*_out.txt` file. For results of the large benchmark sets, there also two `.csv` files with aggregated results, as produced by the runner script.

> Note that the computation times on the `TACAS23 VM` might be a bit higher.

## Benchmark scripts 

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

## Executing prepared benchmarks

There are several pre-configured benchmark commands that you can use. Of course, you can add `-p N` or `-i` to each command involving the runner script to run it in parallel or interactive mode.

Note that executing the full performance benchmarks and parameter scan benchmarks might take a long time (up to several days), because it involves hundreds (thousands) of models.

> Always make sure that the provided Python virtual environment is active before executing the experiments. Execute all the commands from `benchmarks` directory.

#### Benchmarking on fully specified models
You can choose on which set of fully specified models to run the experiments. Experiments with the full benchmark set may take up to tens of hours (without parallelism). This is often caused due to 1H timeouts. The selected 100 model subset contains the benchmarks with fastest computation times, that were tested on `TACAS23 VM`. The total computation times (without parallelism) are summarized by the table:

| experiment | time 230 models | time 100 models |
| -------- | ------- | ------- |
| model checking phi1 | 14H | 15s |
| model checking phi2 | 29H | 20s |
| model checking phi3 | 14H | 15s |
| model checking phi4 | 93H | 4min |
| classification | 94H | 4min |


If you choose to run the benchmark experiments on all 230 models, use the following commands:
```
# model checking for each formula
python3 run.py 1h models/fully-specified-models model-checking-p1.py
python3 run.py 1h models/fully-specified-models model-checking-p2.py
python3 run.py 1h models/fully-specified-models model-checking-p3.py
python3 run.py 1h models/fully-specified-models model-checking-p4.py

# classification
python3 run.py 1h models/fully-specified-models classification.py
```

If you choose to run the benchmark experiments on the subset of selected 100 models, use the following commands:
```
# model checking for each formula
python3 run.py 1h models/fully-specified-models-subset model-checking-p1.py
python3 run.py 1h models/fully-specified-models-subset model-checking-p2.py
python3 run.py 1h models/fully-specified-models-subset model-checking-p3.py
python3 run.py 1h models/fully-specified-models-subset model-checking-p4.py

# classification
python3 run.py 1h models/fully-specified-models-subset classification.py
```


#### Parameter scan benchmarks
Expected times for running parameter scan experiments are given in the table below (without parallelism). The high runtimes are because we are evaluating 1024 instances for each model. To just quickly test a subset, run the experiments for model `132`.

| model | time |
| -------- | ------- |
| 018 | 19H |
| 019 | 5H |
| 056 | 37H | 
| 077 | 93H | 
| 132 | 3min |
| 137 | 17H |
| 207 | 100H |

To run the prepared parameter scan benchmarks, choose model `ID`, substitute it in the command, and execute:

```
# parameter scan for 1024 selected instances of a model with given `ID`
python3 run.py 1h models/parameter-scan/ID model-checking-p2.py
```

The corresponding coloured model-checking experiments take ~30min in total. To run the experiments for all 7 PSBNs, execute:

```
# coloured model checking for all 7 PSBNs
python3 run.py 1h models/parameter-scan/PSBNs model-checking-p2.py
```


#### PSBNs with higher-arity function symbols
To run the prepared benchmarks for PSBN models with higher-arity function symbols, execute the following command. The total computation time should be less than 10min.
```
# classification for all 5 PSBNs
python3 run.py 1h models/large-parametrized-models classification.py
```

## Running experiments on selected models

You can also directly execute scripts `classification.py` or `model-checking.py` on selected models to run single experiments. To construct own formulae, choose from the ones provided in plaintext in `plain-text-formulae.txt` or see the tool's Manual.

```
python3 classification.py MODEL_PATH
python3 model-checking.py MODEL_PATH --formula HCTL_FORMULA
```

In particular, the following experiments should all successfully finish in `under 1 minute` when run on a `TACAS 23 VM`:

1) model checking and classification experiments on fully specified model `206` with 41 variables
```
python3 classification.py models/fully-specified-models/206.aeon

python3 model-checking.py models/fully-specified-models/206.aeon --formula '3{x}: (@{x}: (AX (~{x} & AF {x})))'
```

2) classification experiment on a PSBN with 18 variables and 50 parameters
```
python3 classification.py models/large-parametrized-models/[MAPK]-[18v]-[50p].aeon
```

3) model-checking experiment on a PSBN with 28 variables and 72 parameters
```
python3 model-checking.py models/large-parametrized-models/[FA-BRCA-pathway]-[28v]-[72p].aeon --formula '3{x}: 3{y}: ((@{x}: ~{y} & AX {x}) & (@{y}: AX {y}))'
```