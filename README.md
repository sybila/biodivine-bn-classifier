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

#### Input model and properties

The input for the analysis is an [AEON model](https://biodivine.fi.muni.cz/aeon) annotated with HCTL formulas.
The details of the HCTL syntax can be found [here](https://github.com/sybila/biodivine-hctl-model-checker).
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
using the `hctl-explorer` command. 

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

#### Example

To run the prepared example, execute the following commands:

```
bn-classifier --output-zip expample-result.zip ./benchmarks/mapk/model-with-properties.aeon
hctl-explorer ./example-result.zip
```

## Development guide

To build the application manually, you will need to install Rust compiler from
[rust-lang.org](https://rust-lang.org) (default installation settings should
work fine). 

Afterwards, you also need to install [tauri](https://tauri.app/) using 
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