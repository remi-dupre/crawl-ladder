import asyncio
import aiohttp


async def sleep_and_print(text: str):
    async with aiohttp.ClientSession() as session:
        await session.get("http://perdu.com")

    print(text)


async def main():
    await asyncio.gather(
        sleep_and_print("a"),
        sleep_and_print("b"),
        sleep_and_print("c"),
        sleep_and_print("d"),
    )


asyncio.run(main())
