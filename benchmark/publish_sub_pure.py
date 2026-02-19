from amqp_client_python import Config, Options, AsyncEventbusRabbitMQ

import asyncio
from time import thread_time_ns


async def run():
    options = Options(queue_name='test_queue', rpc_exchange_name='test_exchange', rpc_queue_name='test_rpc_queue')
    config = Config(options=options)
    eventbus = AsyncEventbusRabbitMQ(
        config, None,
        pub_publisher_confirms=True, rpc_client_publisher_confirms=True, rpc_server_publisher_confirms=False, sub_auto_ack=True, rpc_server_auto_ack=True, rpc_client_auto_ack=True)
    await asyncio.sleep(1)
    exchange_name = options.rpc_exchange_name
    routing_key = "abc.example"
    def handler(body):
        pass #print(f"Received message: {body}")
    #    return body
    #await eventbus.rpc_server(handler, routing_key, "application/json", None)
    await eventbus.subscribe(exchange_name, routing_key, handler)
    await asyncio.sleep(10)
    before = thread_time_ns()
    #await eventbus.dispose()
    
    for _ in range(0, 30):
        for _ in range(0, 10000):
        #print("response:", await eventbus.rpc_client(exchange_name, routing_key, b'Hello, RPC!', "application/json", 100_000, None, None))
            print("response:", await eventbus.publish(exchange_name, routing_key, 'Hello, RPC!', "application/json"))
        #await asyncio.sleep(1)
        #'''
    after = thread_time_ns()
    print(f"Time taken for 300k messages: {(after - before) / 1_000_000_000} seconds")
    print(f"Mean messages per second for 300k messages: {300_000 / ((after - before) / 1_000_000_000)}")
    await eventbus.dispose(False)
    #Time taken for 300k messages: 104.610308301 seconds
    #Mean messages per second for 300k messages: 2867.7862141156907
asyncio.run(run())