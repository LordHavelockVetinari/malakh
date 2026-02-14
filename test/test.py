import os
import subprocess
from subprocess import PIPE, DEVNULL
import textwrap
from threading import Thread, Lock
import queue
from queue import Queue

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

def do_job(name):
    progfile = name + ".prog"
    input = read(name + ".in")
    output = read(name + ".out")
    error = read(name + ".err")
    completed = subprocess.run(
        [EXECUTABLE, progfile],
        input=input,
        stdout=PIPE,
        stderr=PIPE,
    )
    with lock:
        if completed.stderr != error:
            print("ERROR: test", repr(name), "returned wrong stderr")
            if error == b"":
                print("EXPECTED: no error")
            else:
                print("EXPECTED:")
                print_indented(error)
            print("GOT:")
            print_indented(completed.stderr)
        elif completed.stdout != output:
            print("ERROR: test", repr(name), "returned wrong output")
            if output == b"":
                print("EXPECTED: no output.")
            else:
                print("EXPECTED:")
                print_indented(output)
            print("GOT:")
            print_indented(completed.stdout)
        else:
            print("OK: test", repr(name), "passed")
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
    dir = os.path.join(os.path.dirname(__file__), "success")
    os.chdir(dir)
    for progfile in os.listdir(dir):
        name, ext = os.path.splitext(progfile)
        if ext != ".prog":
            continue
        num_tests += 1
        job_queue.put(name)
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
