import requests
from collections import deque

from utils import server

MAX_QUERIES = 100

session = requests.Session()  # Initialisation
session.headers["X-User"] = "remi-dupre"  # Auth
urls_to_fetch = deque([server.BASE_URL + "/crawl/"])

for idx in range(MAX_QUERIES):
    # Choisi un URL qui n'a jamais été ouverte
    url = urls_to_fetch.popleft()
    print(f"Request {idx:03}: {url}")
    # Appelle le serveur
    resp = session.get(url).json()
    # Collecte les nouvelles URLs
    urls_to_fetch.extend(child["url"] for child in resp["children"])
