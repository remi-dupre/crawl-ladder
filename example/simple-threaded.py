import functools
import threading
from collections import deque

import requests

from utils import server

NB_THREADS = 320
MAX_QUERIES = 32000


def run_thread(thread_idx: int, nb_thread_queries: int):
    session = requests.Session()
    session.headers["X-User"] = "remi-dupre"
    urls_to_fetch = deque([server.BASE_URL + "/crawl/"])

    for idx in range(nb_thread_queries):
        # Choisi un URL qui n'a jamais été ouverte
        url = urls_to_fetch.popleft()
        # print(f"(thread {thread_idx}) Request {idx:03}: {url}")
        # Appelle le serveur
        resp = session.get(url).json()
        # Collecte les nouvelles URLs
        urls_to_fetch.extend(child["url"] for child in resp["children"])


threads = [
    threading.Thread(
        target=functools.partial(
            run_thread,
            thread_idx=thread_idx,
            nb_thread_queries=MAX_QUERIES // NB_THREADS,
        )
    )
    for thread_idx in range(NB_THREADS)
]

# Start all threads
for thread in threads:
    thread.start()

# Wait for all threads to finish
for thread in threads:
    thread.join()
