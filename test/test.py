import os
import subprocess
from subprocess import PIPE, DEVNULL
import textwrap
from threading import Thread, Lock
import queue
from queue import Queue
from collections import namedtuple

Job = namedtuple("Job", ["dir", "name"])

EXECUTABLE = os.path.join(
    os.path.dirname(os.path.dirname(__file__)),
    "target",
    "debug",
    "malakh.exe"
)

def read(filename):
    try:
        with open(filename, "rb") as f:
            return f.read().replace(b"\r\n", b"\n")
    except IOError:
        return b""

lock = Lock()
num_successful = 0
job_queue = Queue()

def print_indented(b: bytes):
    print(textwrap.indent(b.decode("utf-8", errors="replace"), " " * 4))

dir_options = {
    "success": ["--raw-errors"],
    "compilation_error": ["--check"]
}

def do_job(job):
    malfile = os.path.join(job.dir, job.name + ".mal")
    input = read(os.path.join(job.dir, job.name + ".in"))
    output = read(os.path.join(job.dir, job.name + ".out"))
    error = read(os.path.join(job.dir, job.name + ".err"))
    completed = subprocess.run(
        [EXECUTABLE, malfile, *dir_options[job.dir]],
        input=input,
        stdout=PIPE,
        stderr=PIPE,
    )
    with lock:
        if job.dir == "compilation_error" and completed.returncode != 1:
            if completed.returncode == 0:
                print("ERROR: test", malfile, "unexpectedly compiled")
            else:
                print("ERROR: test", malfile, "compilation failed with exit code", completed.returncode)
        elif job.dir == "success" and completed.stderr != error:
            print("ERROR: test", malfile, "returned wrong stderr")
            if error == b"":
                print("EXPECTED: no error")
            else:
                print("EXPECTED:")
                print_indented(error)
            print("GOT:")
            print_indented(completed.stderr)
        elif job.dir == "success" and completed.stdout != output:
            print("ERROR: test", malfile, "returned wrong output")
            if output == b"":
                print("EXPECTED: no output.")
            else:
                print("EXPECTED:")
                print_indented(output)
            print("GOT:")
            print_indented(completed.stdout)
        else:
            print("OK: test", malfile, "passed")
            global num_successful
            num_successful = num_successful + 1

def worker():
    while True:
        try:
            job = job_queue.get()
        except queue.ShutDown:
            return
        do_job(job)
        job_queue.task_done()

def main():
    global num_successful
    num_tests = num_successful = 0
    subprocess.run("cargo build", check=True, stderr=DEVNULL)

    os.chdir(os.path.dirname(__file__))
    for dir in ["success", "compilation_error"]:
        for malfile in os.listdir(dir):
            name, ext = os.path.splitext(malfile)
            if ext != ".mal":
                continue
            num_tests += 1
            job_queue.put(Job(dir, name))

    job_queue.shutdown()
    for i in range(os.cpu_count()):
        Thread(target=worker).start()
    job_queue.join()

    print()
    if num_successful == num_tests:
        print(f"All {num_tests} tests passed!")
    else:
        print(f"{num_successful}/{num_tests} tests passed")

main()
