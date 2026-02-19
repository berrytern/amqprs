from pyexpat.errors import messages
from amqp_rs import Config, ConfigOptions, AsyncEventbus, QoSConfig

import asyncio
from time import thread_time_ns


async def run():
    options = ConfigOptions(queue_name='test_queue', rpc_exchange_name='test_exchange', rpc_queue_name='test_rpc_queue')
    config = Config(host='localhost', port=5672, username='guest', password='guest', virtual_host='/', options=options)
    eventbus = AsyncEventbus(config, QoSConfig(pub_confirm=True, rpc_client_confirm=True, rpc_server_confirm=True, sub_auto_ack=True, rpc_server_auto_ack=True, rpc_client_auto_ack=True, sub_prefetch=None, rpc_server_prefetch=None, rpc_client_prefetch=None))
    await asyncio.sleep(1)
    exchange_name = options.rpc_exchange_name
    routing_key = "abc.example"
    async def handler(body):
        print(f"Received message: {body}")
        return body
    await eventbus.rpc_server(routing_key, handler, None, None)
    await asyncio.sleep(10)
    before = thread_time_ns()
    
    for _ in range(0, 30):
        for _ in range(0, 10000):
            print("response:", await eventbus.rpc_client(exchange_name, routing_key, b'Hello, RPC!', 'application/json', 50_000, 100, None))
            #print("response:", await eventbus.publish(exchange_name, routing_key, 'Hello, RPC!', "application/json"))
        #await asyncio.sleep(1)
        #'''
    after = thread_time_ns()
    print(f"Time taken for 300k messages: {(after - before) / 1_000_000_000} seconds")
    print(f"Mean messages per second for 300k messages: {300_000 / ((after - before) / 1_000_000_000)}")
    await eventbus.dispose()
    # Time taken for 300k messages: 50.411525485 seconds
    # Mean messages per second for 300k messages: 5951.020071575999
asyncio.run(run())