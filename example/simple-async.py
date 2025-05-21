import asyncio
from collections import deque

import aiohttp

from utils import server

NB_TASKS = 640
MAX_QUERIES = 6400


async def run_task(task_idx: int, nb_task_queries: int):
    async with aiohttp.ClientSession(headers={"X-User": "remi-dupre"}) as session:
        urls_to_fetch = deque([server.BASE_URL + "/crawl/"])

        for idx in range(nb_task_queries):
            # Choisi un URL qui n'a jamais été ouverte
            url = urls_to_fetch.popleft()
            # print(f"(task {task_idx}) Request {idx:03}: {url}")
            # Appelle le serveur
            resp = await session.get(url)
            resp = await resp.json()
            # Collecte les nouvelles URLs
            urls_to_fetch.extend(child["url"] for child in resp["children"])


async def main():
    tasks = [
        asyncio.create_task(
            run_task(
                task_idx=task_idx,
                nb_task_queries=MAX_QUERIES // NB_TASKS,
            )
        )
        for task_idx in range(NB_TASKS)
    ]

    for task in tasks:
        await task


asyncio.run(main())
