from biodivine_aeon import *
from pathlib import Path
from argparse import ArgumentParser


# Run classification analysis
def classify(model_path):
    print("Model path", model_path, flush=True)
    output_path = model_path.split(".")[0] + "-archive.zip"
    run_hctl_classification(model_path, output_path)


if __name__ == "__main__":
    # Create command-line arguments parser
    parser = ArgumentParser(description="Model check HCTL formula on a given PSBN model.")
    parser.add_argument("model_path", type=str, help="Path to a file with model and properties.")

    # Parse the command-line arguments
    args = parser.parse_args()

    # Run the actual classification
    classify(args.model_path)
