import asyncio
import itertools
import os

import aiohttp

from utils import server

MAX_PARALLEL = int(os.getenv("MAX_PARALLEL", 10000))


async def get_children(http: aiohttp.ClientSession, url: list[str]) -> list[str]:
    async with http.get(url) as resp:
        data = await resp.json()

    return [child["url"] for child in data["children"]]


async def crawl(http: aiohttp.ClientSession, urls: list[str]) -> list[str]:
    tasks = (get_children(http, url) for url in urls)
    return list(itertools.chain(*(await asyncio.gather(*tasks))))


async def main():
    urls = [server.BASE_URL + "/crawl/"]

    for _ in range(10**4):
        async with aiohttp.ClientSession(
            headers={"X-User": os.getenv("USER", "remi-dupre")},
            proxy="http://localhost:3128",
        ) as http:
            print(f"Fetch {len(urls)} urls")
            urls = await crawl(http, urls)
            urls = urls[:MAX_PARALLEL]


if __name__ == "__main__":
    asyncio.run(main())
