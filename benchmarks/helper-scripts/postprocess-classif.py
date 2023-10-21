import argparse
from pathlib import Path


def fix_results(source_directory,):
    source_path = Path(source_directory)

    # iterate through individual results in dir, remove incorrect results, collect their IDS
    ids_to_remove = []
    for file_name in source_path.iterdir():
        # special treatment for "aggregated" and "times" summaries
        if "aggregated.csv" in file_name.name or "times.csv" in file_name.name:
            continue
        else:
            id = file_name.stem[0:3]
            out_of_mem = False
            timeout = False

            # check for incomplete results
            with file_name.open('r') as f:
                content = f.read()
                # if there are no results and no output from time command -> timeouted correctly
                # (only adjust if there is not a timeout/oom note already)
                if ("Results saved to" not in content and
                    "user" not in content and
                    "Did not finish due to 1h timeout." not in content and
                    "Did not finish due to out-of-memory" not in content
                ):
                    timeout = True
                # if there is something else, but no results -> out of memory
                # (only adjust if there is not a timeout/oom note already)
                elif ("Results saved to" not in content and
                      "user" in content and
                      "Did not finish due to 1h timeout." not in content and
                      "Did not finish due to out-of-memory" not in content
                ):
                    out_of_mem = True
                    # only remove IDs with oom flag, timeouts are correctly handled during computation
                    ids_to_remove.append(id)

            assert not (out_of_mem and timeout)
            if out_of_mem:
                with file_name.open('w') as f:
                    f.write("Did not finish due to out-of-memory.\n")

            if timeout:
                with file_name.open('a') as f:
                    f.write("Did not finish due to 1h timeout.\n")

    # special treatment for "times" summary, and collecting data for aggregation
    times = []
    for file_name in source_path.iterdir():
        if "times.csv" in file_name.name:
            new_content = []
            with file_name.open('r') as f:
                content = f.read()
                for line in content.split("\n"):
                    if not line or "Benchmark, Time[s]" in line:
                        continue
                    name, time = line.split(", ")[0], line.split(", ")[1]

                    if name[0:3] in ids_to_remove:
                        # add info about the fail
                        new_content.append(name + ", " + "fail")
                        continue

                    if time != "fail":
                        times.append(float(time))

                    new_content.append(line)
                new_content = "\n".join(new_content)
            with file_name.open('w') as f:
                f.write("Benchmark, Time[s]\n")
                f.write(new_content + "\n")

    # special treatment for "aggregated" summary
    times = sorted(times)
    for file_name in source_path.iterdir():
        if "aggregated.csv" in file_name.name:
            with file_name.open('w') as f:
                f.write("Time[s], No. Completed\n")
                f.write(str(times[0]) + ", 0\n")
                for i in range(len(times)):
                    f.write(str(times[i]) + ", " + str(i+1) + "\n")

                # we have a 1h timeout.
                f.write("3600, " + str(len(times)) + "\n")

    print(f"Removing results for {len(ids_to_remove)} models with following IDs:", end=" ")
    for id in ids_to_remove:
        print(id, end=" ")
    print()


if __name__ == "__main__":
    # create command-line arguments parser.
    parser = argparse.ArgumentParser(description="Remove unsuccessful benchmark results.")
    parser.add_argument("source_directory", type=str, help="Source directory with result files.")

    args = parser.parse_args()

    fix_results(args.source_directory)
