import multiprocessing


def fibonacci(n: int):
    # !!! Never do this
    return 1 if n <= 1 else fibonacci(n - 1) + fibonacci(n - 2)


def slow_function():
    print("fibonacci(35):", fibonacci(35))


NB_TASKS = 8

procs = [multiprocessing.Process(target=slow_function) for _ in range(NB_TASKS)]

for process in procs:
    process.start()

for process in procs:
    process.join()
