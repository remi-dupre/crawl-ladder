import os

BASE_URL = os.getenv("BASE_URL", "http://crawl.dupre.io:8000")
MAX_PARALLEL = int(os.getenv("MAX_PARALLEL", 10000))
