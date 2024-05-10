import sys
import os
import argparse
import subprocess
import datetime
from tempfile import NamedTemporaryFile
from dataclasses import dataclass

### Settings ###

TIMEOUT = 5
PYTHON_PATH = sys.executable  # always use the current python
IR_PATH = "./ir"
EXECUTOR_PATH = "../target/debug/accipit"

### Color Utils ###


def red(s: str) -> str:
    return f"\033[31m{s}\033[0m"


def green(s: str) -> str:
    return f"\033[32m{s}\033[0m"


def box(s: str) -> str:
    return f"\033[1;7;37m{s}\033[0m"

### Test Utils ###


@dataclass
class Test:
    filename: str
    inputs: list[str] | None
    expected: list[str] | None
    should_fail: bool

    @classmethod
    def parse_file(cls, filename: str):
        content = open(filename).readlines()
        comments = []
        for line in content:
            # get comment, start with //
            if line.startswith("//"):
                comments.append(line[2:])
            else:
                break
        if len(comments) == 0:  # no comment means success
            return cls(filename, None, None, False)
        elif len(comments) == 1:
            if "Error" in comments[0]:  # should fail
                return cls(filename, None, None, True)
            else:
                return cls(filename, None, None, False)
        elif len(comments) == 2:  # input and output
            assert "Input:" in comments[0], f"Error: {filename} has non-paired input/output"
            assert "Output:" in comments[1], f"Error: {filename} has non-paired input/output"
            input = comments[0].replace("Input:", "").strip().split()
            expected = comments[1].replace("Output:", "").strip().split()
            return cls(filename, input, expected, False)
        else:
            assert False, f"Error: {filename} heading comment is invalid"

    def __str__(self):
        return f"Test({self.filename}, {self.inputs}, {self.expected}, {self.should_fail})"


class TestResult:
    def __init__(self, test: Test, output: str | None | list[str], exit_code: int, concat_output: bool = False):
        self.test = test
        self.output = output
        self.exit_code = exit_code
        if test.should_fail:
            self.passed = exit_code != 0
        else:
            if test.expected is None:
                self.passed = exit_code == 0
            else:
                if not concat_output:  # lab3
                    self.passed = exit_code == 0 and output == test.expected
                else:  # lab4
                    expected = "".join(test.expected)
                    self.passed = exit_code == 0 and output == expected


def run_one_test(compiler: str, test: Test, lab: str, local: bool) -> TestResult:
    def run_only_compiler(compiler: str, test: Test) -> TestResult:  # lab1, lab2
        if test.inputs is None:  # no input
            try:
                result = subprocess.run(
                    [compiler, test.filename], capture_output=True, timeout=TIMEOUT)
            except subprocess.TimeoutExpired:
                print(red(f"Error: {test.filename} timed out."))
                return TestResult(test, None, -1)
            # get exit code and output
            exit_code = result.returncode
            output = result.stdout.decode("utf-8")
            return TestResult(test, output, exit_code)
        raise NotImplementedError("Not implemented input for lab1 or lab2")
    
    def run_with_ir(compiler: str, test: Test) -> TestResult:  # lab3
        if not local:
            ir_file = NamedTemporaryFile(suffix=".ll")
            ir_file_name = ir_file.name
        else:
            ir_file_name = test.filename.replace(
                ".sy", ".acc").split("/")[-1]
            ir_file_name = f"{IR_PATH}/{ir_file_name}"
        assert test.expected is not None, f"Error: {test.filename} has no expected output."
        try:
            result = subprocess.run(
                [compiler, test.filename, ir_file_name],
                capture_output=True,
                timeout=TIMEOUT)
            if result.returncode != 0:  # compile error
                return TestResult(test, None, result.returncode)
            with subprocess.Popen([EXECUTOR_PATH, ir_file_name],
                                  stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE,
                                  stderr=subprocess.DEVNULL,
                                  text=True) as p:
                try:
                    if test.inputs is not None:
                        outputs, _ = p.communicate(
                            input="\n".join(test.inputs), timeout=TIMEOUT)
                    else:
                        outputs, _ = p.communicate(timeout=TIMEOUT)
                    outputs = outputs.strip().split("\n")
                    returnvalue = p.returncode
                    if isinstance(outputs, list):
                        outputs = outputs[0].split()
                    return TestResult(test, outputs, returnvalue)
                except subprocess.TimeoutExpired:
                    p.kill()
                    raise
        except subprocess.TimeoutExpired:
            print(red(f"Error: {test.filename} timed out."))
            return TestResult(test, None, -1)

    match lab:
        case "lab1" | "lab2":
            return run_only_compiler(compiler, test)
        case "lab3":
            return run_with_ir(compiler, test)
        case _:
            raise NotImplemented


def summary(test_results: list[TestResult]):
    # get the longest filename
    max_filename = max([len(test_result.test.filename)
                        for test_result in test_results])
    for test_result in test_results:
        # align the filename
        print(f"{test_result.test.filename.ljust(max_filename)}  ", end="")
        print(f"{green('PASSED') if test_result.passed else red('FAILED')}")
    passed = len([test for test in test_results if test.passed])
    print()
    if passed == len(test_results):
        print(green("All tests passed!"))
    else:
        print(f"{passed}/{len(test_results)} tests passed.")


def test_lab(compiler: str, lab: str, local: bool) -> list[TestResult]:
    print(box(f"Running {lab} test..."))
    tests = os.listdir(f"./{lab}")
    tests = filter(lambda x: x.endswith(".sy"), tests)  # only test .sy files
    tests = [Test.parse_file(f"./{lab}/{test}") for test in tests]
    test_results = [run_one_test(compiler, test, lab, local) for test in tests]
    return test_results


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Test your compiler.")
    parser.add_argument("input_file", type=str, help="Your complier file")
    parser.add_argument("lab", type=str, help="Which lab to test",
                        choices=["lab1", "lab2", "lab3", "lab4"])
    parser.add_argument("-l", "--local", action="store_true",
                        help="Generate temporary files locally.")
    parser.add_argument("--executor_path", type=str, default=EXECUTOR_PATH,
                        help="Path to the executor file.")
    args = parser.parse_args()
    input_file, lab, local, EXECUTOR_PATH = args.input_file, args.lab, args.local, args.executor_path
    if not os.path.exists(input_file):
        print(f"File {input_file} not found.")
        exit(1)
    if not os.path.exists(EXECUTOR_PATH):
        print(f"Executor file {EXECUTOR_PATH} not found.")
        exit(1)
    if local:
        if not os.path.exists(IR_PATH):
            os.mkdir(IR_PATH)
    test_results = test_lab(input_file, lab, local)
    summary(test_results)
    print(datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S"))
