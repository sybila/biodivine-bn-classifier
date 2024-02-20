# BN Classifier

BN Classifier is a tool for analysing Boolean networks with partially unknown behaviour through model checking of hybrid temporal properties.
The tool consists of two parts: A command line *classification engine* and a *tree visualizer* GUI application.

## Distribution

We provide pre-built binaries for both tool components in the [release section](https://github.com/sybila/biodivine-bn-classifier/releases)
(includes versions for Windows, Linux and macOS). Alternatively, if you want to build the tool locally, the instructions are provided below
in the *Development guide*.

To start using BN Classifier, download `bn-classifier` binary for your operating system (this is the command line classification engine), 
as well as the `hctl-explorer` application (this is the graphical user interface). For the `bn-classifier`, there are only three options
based on your OS. For the `hctl-explorer`, you can choose between `.app` and `.dmg` for macOS, `.AppImage` or `.deb` for Linux,
or `.msi` for Windows.

 > Note that the binaries are not signed, so macOS and Windows will likely ask if you trust the application or otherwise 
 require you to explicitly allow it to run. We do not include instructions for these steps.

In this readme, we use `bn-classifier` and `hctl-explorer` as names for the two components you downloaded, even though the binaries
can have slightly different names depending on what OS you are using. We also assume they are present in your PATH. If that is
not the case, please substitute them for the full path to the downloaded binary.

## Using BN Classifier

#### Manual

In the `manual.pdf`, we provide detailed instructions on 
- installation,
- input format,
- running the classification engine,
- running the GUI (with illustrations).

Below, we briefly summarize some of the functionality.


#### Input model and properties

The input for the analysis is an [AEON model](https://biodivine.fi.muni.cz/aeon) annotated with HCTL formulas.
The details of the HCTL syntax can be found in the `manual.pdf`.
To get familar with AEON models, we recommend [this page](https://biodivine.fi.muni.cz/aeon/manual/v0.4.0/model_editor/import_export.html) 
of the AEON manual. A wide range of real-world models for testing can be also found 
[here](https://github.com/sybila/biodivine-boolean-models) (however, these models do not contain any HCTL formulas).

Each HCTL formula represents either an *assertion*, or a *named property*. The assertions restrict the space of
possible network parametrisations: the tool will disregard any parametrisation that does not satisfy all
assertions (formula satisfaction is *universal*, i.e. in order to be satisfied, it must be satisfied in all states). 
Meanwhile, properties are used for the classification step. Here, the tool will enumerate unique 
groups of properties that can hold together and the parametrisations where this occurs (again, a formula holds if 
it holds in all states). The relationships between these groups can be then explored through the decision 
tree visualization tool.

Below, we show a simple example of how to include assertions and properties in a model file (we intentionally
limit the use of hybrid operators to simplify the example). You can find  additional examples that combine 
hybrid assertions and properties in complex models in the `benchmarks` directory. 

```
# This property states that every state must be able to reach a state where
# variable `Apoptosis` is true. `#!` is used to start a "comment annotation"
# The #` and `# serve as opening/closing escape characters for the HCTL formula.
#! dynamic_assertion: #`EF Apoptosis`#

# This property states that there must be a state in which `Apoptosis`
# holds, and this state is a fixed-point.
#! dynamic_assertion: #`3{x}: @{x}: (Apoptosis & AX x)`#

# Consequently, the classification step will only consider parametrisations
# that satisfy the two assertions. To further disambiguate between 
# parametrisations, we can define named properties:

# Property `will_die` states that the system will always eventually reach `Apoptosis`.
#! dynamic_property: will_die: #`AF AG Apoptosis`#

# Property `cannot_be_undead` states that whenever `Apoptosis` is true, it must
# stay true forver.
#! dynamic_property: cannot_be_undead: #`Apoptosis => AG Apoptosis`#
```

#### Running classification

Once you have an `annotated-model.aeon` file, you can run the classification engine:

`bn-classifier --output-zip /path/to/output-archive.zip path/to/annotated-model.aeon`

The output is written into the `output-archive.zip` which contains both a plaintext
`report.txt` where you can see a summary of the results, as well as raw BDD dumps
(compatible with the [lib-bdd](https://github.com/sybila/biodivine-lib-bdd) string 
representation) that can be imported into the `hctl-explorer`.

#### Running visualisation

Once you obtain the classification results, you can run the visualisation tool
using the `hctl-explorer` command. If you simply start `hctl-explorer` without
any additional arguments, you will be prompted to select the result archive that 
you want to explore. Alternatively, you can also give `hctl-explorer` 
a path to the archive as a command line argument:

`hctl-explorer path/to/classification_result.zip`

This command should open a window with an interactive decision tree editor.
In this editor, you can branch the set of results by conditioning on various
model parameters. Once only a single outcome (combination of valid properties)
is admissible, the part of the tree will be shown as a leaf. You can either
choose the branching conditions manually, or let the tool infer a suitable
tree based on information entropy of the dataset.

For any leaf node, you can save a canonical witness Boolean network (i.e.
a network with all parameters set to a fixed value). You can also randomly
sample the space of networks that appear in this leaf node.

#### Simple example

To run the prepared example, execute the following commands:

```
bn-classifier --output-zip example-result.zip ./tutorial/case-study-mapk/mapk-annotated.aeon
hctl-explorer ./example-result.zip
```

## Tutorial and Benchmarks

We provide a tutorial and a set of benchmarks to experiment with. Both of them utilize the Python bindings to the Rust code of the classification engine. These are provided as part of our standard library [aeon.py](https://github.com/sybila/biodivine-aeon-py/).

Follow the instructions below to 1) install all dependencies and 2) run the tutorial or execute benchmarks.

#### Creating the Python `venv` and installing dependecies

First, we create a Python virtual environment. For this, we assume you have Python 3 installed with support for virtual environments (current minimal supported version is Python 3.9). For example, on debian-based linux, this corresponds to system packages `python3`, `python3-pip` and `python3-venv` (i.e. `sudo apt install python3 python3-pip python3-venv`).

Then run the following to create a fresh Python environment with all the dependencies:
1. `python3 -m venv ./env` to create a new blank environment in the `env` folder.
2. Next, run `source ./env/bin/activate` to activate the environment. 
3. Finally, run `pip3 install -r requirements.txt` to install all the relevant dependencies.

You will need to have this environment active to run both the benchmarks and tutorial (just run `source ./env/bin/activate` in correct directory to enable the virtual environment). To deactivate the environment, simply execute the `deactivate` command.


#### Running the tutorial notebook

To open the jupyter notebook environment: `python3 -m jupyter notebook --no-browser`

This should start a jupyter notebook environment that you can access through the web browser. You need to find a "Or copy and paste one of these URLs:" message. Then copy the URL under this message and paste it into a browser window (Firefox is installed in the VM). The URL should look like this: `http://localhost:8888/tree?token=SOME_RANDOM_STRING_OF_CHARACTERS`. This will open a jupyter notebook user interface. You can keep this window open, because it will be used in the tutorial.

Navigate to the `tutorial` folder and open the `tutorial.ipynb`  notebook. The notebook contains all the relevant instructions on how to use *BN Classifier* as a Python library as well as the explorer GUI.  

>  If you are unfamiliar with Jupyter notebooks, you can execute a code cell by selecting it and pressing `Shift+Enter`. Alternatively, you can also press the "run" button in the top toolbar (using the "play" arrow icon).

#### Running the benchmarks

All the details regarding the benchmarking process, the models we use, and pre-computed results are provided in `./benchmarks/README.md`.


## Development guide

To build the application manually, you will need to install Rust compiler from
[rust-lang.org](https://rust-lang.org) (default installation settings should
work fine). 

Afterward, you also need to install [tauri](https://tauri.app/) using 
`cargo install tauri-cli`.

#### Compiling `bn-classifier`

To compile the classification engine, go to the `classifier` folder, and
execute `cargo build --release`. This should place a `bn-classifier` binary
into the `target/release` directory.

Alternatively, to run the classifier directly, you can use (still in the `classifier` directory):

```
cargo run --release --bin bn-classifier -- path/to/annotated-model.aeon path/to/output.zip
```

#### Compiling `hctl-explorer`

To compile the visualisation GUI, stay in the repository root folder,
and execute `cargo tauri build`. The application binaries/bundles should
then appear in `src-tauri/target/release` (the specific type of binaries/bundles
depends on your OS).

To run the application directly, you can use (still in the repository root):

```
cargo tauri dev -- -- path/to/classification-results.zip
```

However, note that the application will execute from the `src-tauri` folder, so the arguments
need to be either absolute paths, or paths relative to the `src-tauri` folder. Furthermore,
in this mode, the app is compiled without optimizations, so the interface can be slow for even 
moderately-sized models. This mode should allow you to use standard 
JavaScript developer console to debug/inspect the UI.
