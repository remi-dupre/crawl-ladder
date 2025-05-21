import functools
import multiprocessing
from collections import deque

import requests

from utils import server

NB_THREADS = 320
MAX_QUERIES = 32000


def run_process(process_idx: int, nb_process_queries: int):
    session = requests.Session()
    session.headers["X-User"] = "remi-dupre"
    urls_to_fetch = deque([server.BASE_URL + "/crawl/"])

    for idx in range(nb_process_queries):
        # Choisi un URL qui n'a jamais été ouverte
        url = urls_to_fetch.popleft()
        # print(f"(process {process_idx}) Request {idx:03}: {url}")
        # Appelle le serveur
        resp = session.get(url).json()
        # Collecte les nouvelles URLs
        urls_to_fetch.extend(child["url"] for child in resp["children"])


procs = [
    multiprocessing.Process(
        target=functools.partial(
            run_process,
            process_idx=process_idx,
            nb_process_queries=MAX_QUERIES // NB_THREADS,
        )
    )
    for process_idx in range(NB_THREADS)
]

# Start all procs
for process in procs:
    process.start()

# Wait for all procs to finish
for process in procs:
    process.join()
