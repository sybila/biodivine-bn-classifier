import sys
import os

from biodivine_aeon import *
from pathlib import Path


annotation_string = """
# universally reachable steady states
#! dynamic_property: p1: #`3{x}: V{y}: (@{y}: EF ({x} & (!{z}: AX {z})))`#

# pair of steady states
#! dynamic_property: p2: #`3{x}: 3{y}: ((@{x}: ~{y} & AX {x}) & (@{y}: AX {y}))`#

# states allowing to reach 2 attractors
#! dynamic_property: p3: #`3{x}: 3{y}: (@{x}: (EF ({y} & (!{z}: AX {z})) & EF (~{y} & (!{z}: AX {z}))))`#

# "checkpoint" states
#! dynamic_property: p4: #`3{x}: (@{x}: (AX (~{x} & AF {x})))`#

"""


# Utility function to check if a given path is a benchmark model.
def is_bench(file):
    return file.endswith(".aeon")


if __name__ == "__main__":
    input_dir = sys.argv[1].strip("/")
    print("Directory with models:", input_dir)

    output_dir = input_dir + "-annotated-4-props"
    os.makedirs(output_dir, exist_ok=True)

    model_files = filter(is_bench, os.listdir(input_dir))
    model_files = sorted(model_files)

    for model_file in model_files:
        output_file = model_file.split(".")[0] + "-annotated.aeon"

        print("Annotating model:", model_file)
        model_string = Path(f"{input_dir}/{model_file}").read_text()

        # append annotation string
        model_string = annotation_string + model_string

        Path(f"{output_dir}/{output_file}").write_text(model_string)
        print("Saving output to:", f"{output_dir}/{output_file}")

