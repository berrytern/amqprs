import pytest
from amqp_rs import AsyncEventbus, Config, ConfigOptions, QoSConfig, ContentEncoding, TlsAdaptor
from asyncio import Future, get_running_loop


@pytest.mark.asyncio
async def test_provider():
    options = ConfigOptions(queue_name='test_queue', rpc_exchange_name='test_exchange', rpc_queue_name='test_rpc_queue')
    eventbus = AsyncEventbus(Config(host='localhost', port=5672, username='guest', password='guest', virtual_host='/', options=options, tls_adaptor=None), QoSConfig(pub_confirm=True, rpc_client_confirm=True, rpc_server_confirm=True, sub_auto_ack=True, rpc_server_auto_ack=True, rpc_client_auto_ack=True, sub_prefetch=None, rpc_server_prefetch=None, rpc_client_prefetch=None))
    expected_result = "received message"
    future = Future(loop = get_running_loop())
    async def handle(body):
        future.set_result(expected_result)
        return "hello"
    routing_key = "abc.example"
    await eventbus.rpc_server(routing_key, handle)
    result = await eventbus.rpc_client(options.rpc_exchange_name, routing_key, ["hi"], "application/json", ContentEncoding.Null)
    assert future.done()
    assert future.result() == expected_result
    assert result == b"hello"
    #await sleep(3)
    
