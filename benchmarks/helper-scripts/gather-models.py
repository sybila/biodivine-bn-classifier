import argparse
import re
from pathlib import Path
import shutil


# Define a function to extract ID from subdirectory name using regex.
def extract_id(directory_name):
    match = re.search(r'\[id-(\d+)\]', directory_name)
    if match:
        return match.group(1)
    else:
        return None

# Define a function to gather model files.
def gather_model_files(source_directory, target_directory):
    source_path = Path(source_directory)
    target_path = Path(target_directory)

    # Create the target directory if it doesn't exist.
    target_path.mkdir(parents=True, exist_ok=True)

    # Iterate through subdirectories in the source directory.
    for subdirectory in source_path.iterdir():
        if subdirectory.is_dir():
            model_file_path = subdirectory / 'model.aeon'
            if model_file_path.is_file():
                # Extract the ID from the subdirectory name.
                ID = extract_id(subdirectory.name)
                if ID is not None:
                    # Create the new filename based on ID.
                    new_filename = f'{ID}.aeon'

                    # Copy the model file to the target directory with the new name.
                    target_file_path = target_path / new_filename
                    shutil.copy2(model_file_path, target_file_path)
                    print(f"Copied {model_file_path} to {target_file_path}")

    print("Model files have been gathered in the target directory.")

if __name__ == "__main__":
    # Create command-line arguments parser.
    parser = argparse.ArgumentParser(description="Gather model files from subdirectories.")
    parser.add_argument("source_directory", type=str, help="Source directory containing subdirectories with model files.")
    parser.add_argument("target_directory", type=str, help="Target directory to gather model files.")

    # Parse the command-line arguments.
    args = parser.parse_args()

    # Call the gather_model_files function with the provided source and target directories.
    gather_model_files(args.source_directory, args.target_directory)

