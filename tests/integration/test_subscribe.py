import pytest
from amqpr import AsyncEventbus, Config, ConfigOptions, QoSConfig
from asyncio import Future, sleep, get_running_loop
from json import dumps




@pytest.mark.asyncio
async def test_subscribe():
    options = ConfigOptions(queue_name='test_queue', rpc_exchange_name='test_exchange', rpc_queue_name='test_rpc_queue')
    eventbus = AsyncEventbus(Config(host='localhost', port=5672, username='guest', password='guest', virtual_host='/', options=options, tls_adaptor=None), QoSConfig(pub_confirm=True, rpc_client_confirm=True, rpc_server_confirm=True, sub_auto_ack=True, rpc_server_auto_ack=True, rpc_client_auto_ack=True, sub_prefetch=None, rpc_server_prefetch=None, rpc_client_prefetch=None))
    expected_result = "received message"
    future = Future(loop = get_running_loop())

    async def handle(_):
        if not future.done():
            future.set_result("received message")

    exchange_name = "example"
    routing_key = "abc.example"
    body = dumps(["hi"])
    await eventbus.subscribe(exchange_name, routing_key, handle)
    await eventbus.publish(exchange_name, routing_key, body)
    await future
    assert future.done()
    assert future.result() == expected_result
    await eventbus.dispose()