from amqp_rs import Config, ConfigOptions, AsyncEventbus, QoSConfig

import asyncio


async def run():
    options = ConfigOptions(queue_name='test_queue', rpc_exchange_name='test_exchange', rpc_queue_name='test_rpc_queue')
    config = Config(host='localhost', port=5672, username='guest', password='guest', options=options)
    eventbus = AsyncEventbus(config, QoSConfig(pub_confirm=True, rpc_client_confirm=True, rpc_server_confirm=True, sub_auto_ack=True, rpc_server_auto_ack=True, rpc_client_auto_ack=True, sub_prefetch=None, rpc_server_prefetch=None, rpc_client_prefetch=None))
    try:
        await asyncio.sleep(1)
        exchange_name = options.rpc_exchange_name
        routing_key = "abc.example"
        def handler(body):
            print(f"Received message: {body}")
            return body
        #await eventbus.rpc_server(handler, routing_key, "application/json", None)
        
        while True:
            #print("response:", await eventbus.rpc_client(exchange_name, routing_key, b'Hello, RPC!', "application/json", 100_000, None, None))
            await eventbus.publish(exchange_name, routing_key, b'Hello, RPC!', "application/json", None)
    except KeyboardInterrupt:
        await eventbus.dispose()
asyncio.run(run())
