from biodivine_aeon import *

input_path = "./tutorial/case-study-mapk/mapk-annotated.aeon"
output_path = "./tutorial/classification-archive.zip"

bn = BooleanNetwork.from_file(input_path)
print(bn)

run_hctl_classification(input_path, output_path)