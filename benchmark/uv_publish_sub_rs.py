from amqp_rs import Config, ConfigOptions, AsyncEventbus, QoSConfig

import asyncio
import uvloop
uvloop.install()
from time import thread_time_ns
from json import dumps


async def run():
    options = ConfigOptions(queue_name='test_queue', rpc_exchange_name='test_exchange', rpc_queue_name='test_rpc_queue')
    config = Config(host='localhost', port=5672, username='guest', password='guest', virtual_host='/', options=options)
    eventbus = AsyncEventbus(config, QoSConfig(pub_confirm=True, rpc_client_confirm=True, rpc_server_confirm=True, sub_auto_ack=True, rpc_server_auto_ack=True, rpc_client_auto_ack=True, sub_prefetch=None, rpc_server_prefetch=None, rpc_client_prefetch=None))
    await asyncio.sleep(1)
    exchange_name = options.rpc_exchange_name
    routing_key = "abc.example"
    async def handler(body):
        pass #print(f"Received message: {body}")
    #    return body
    #await eventbus.rpc_server(handler, routing_key, "application/json", None)
    await eventbus.subscribe(exchange_name, routing_key, handler, None, None)
    await asyncio.sleep(10)
    before = thread_time_ns()
    #await eventbus.dispose()
    
    for _ in range(0, 30):
        for _ in range(0, 10000):
        #print("response:", await eventbus.rpc_client(exchange_name, routing_key, b'Hello, RPC!', "application/json", 100_000, None, None))
            #print("response:", await eventbus.publish(exchange_name, routing_key, bytes(dumps('Hello, RPC!'), "utf-8"), "application/json", None))
            await eventbus.publish(exchange_name, routing_key, bytes(dumps('Hello, RPC!'), "utf-8"), "application/json", None)
        #await asyncio.sleep(1)
        #'''
    
    after = thread_time_ns()
    print(f"Time taken for 300k messages: {(after - before) / 1_000_000_000} seconds")
    print(f"Mean messages per second for 300k messages: {300_000 / ((after - before) / 1_000_000_000)}")
    #Time taken for 300k messages: 25.522518368 seconds
    #Mean messages per second for 300k messages: 11754.325951475792
    #
    await eventbus.dispose()
asyncio.run(run())