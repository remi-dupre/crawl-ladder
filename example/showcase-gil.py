import threading


def fibonacci(n: int):
    # !!! Never do this
    return 1 if n <= 1 else fibonacci(n - 1) + fibonacci(n - 2)


def slow_function():
    print("fibonacci(35):", fibonacci(35))


NB_TASKS = 2

threads = [threading.Thread(target=slow_function) for _ in range(NB_TASKS)]

for thread in threads:
    thread.start()

for thread in threads:
    thread.join()
