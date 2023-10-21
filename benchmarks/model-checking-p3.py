from argparse import ArgumentParser
from biodivine_aeon import *
from pathlib import Path


# Run model-checking analysis
def run_model_checking(model_path, formula):
    print("Model path", model_path, flush=True)

    model_string = Path(model_path).read_text()
    model = BooleanNetwork.from_aeon(model_string)

    print("Loaded model with", model.num_vars(), "variables.", flush=True)

    mc_analysis(model, formula, print_progress=False)


if __name__ == "__main__":
    # Create command-line arguments parser
    default_formula = "3{x}: 3{y}: (@{x}: (EF ({y} & (!{z}: AX {z})) & EF (~{y} & (!{z}: AX {z}))))"

    parser = ArgumentParser(description="Model check HCTL formula on a given PSBN model.")
    parser.add_argument("model_path", type=str, help="Path to the file with model in AEON format.")
    parser.add_argument('--formula', default=default_formula, help=f'Formula to model check. Default is `{default_formula}`.')

    # Parse the command-line arguments
    args = parser.parse_args()

    # Run the actual model checker
    run_model_checking(args.model_path, args.formula)
