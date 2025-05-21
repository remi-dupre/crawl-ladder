import asyncio


async def some_work():
    print("started")
    await asyncio.sleep(1)
    print("finished")


async def main():
    some_work()


asyncio.run(main())
