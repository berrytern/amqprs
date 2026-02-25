import pytest
from amqpr import AsyncEventbus, Config, ConfigOptions, QoSConfig, ContentEncoding
from asyncio import Future, sleep, get_running_loop
from json import dumps




@pytest.mark.asyncio
async def test_subscribe():
    future = Future(loop = get_running_loop())
    options = ConfigOptions(queue_name='test_queue', rpc_exchange_name='test_exchange', rpc_queue_name='test_rpc_queue')
    eventbus = AsyncEventbus(Config(host='localhost', port=5672, username='guest', password='guest', virtual_host='/', options=options, tls_adaptor=None), QoSConfig(pub_confirm=True, rpc_client_confirm=True, rpc_server_confirm=True, sub_auto_ack=True, rpc_server_auto_ack=True, rpc_client_auto_ack=True, sub_prefetch=None, rpc_server_prefetch=None, rpc_client_prefetch=None))
    async def handle(body: bytes):
        if not future.done():
            future.set_result(body)

    exchange_name = "test"
    routing_key = "test.action"
    message = b"received message"
    await eventbus.subscribe(exchange_name, routing_key, handle, None, None)
    await eventbus.publish(exchange_name, routing_key, message, None, ContentEncoding.Null, None)
    await future
    assert future.done()
    assert future.result() == message
    await eventbus.dispose()