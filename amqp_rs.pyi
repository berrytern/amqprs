from typing import Callable, Optional, Any, List, Awaitable, Union
from enum import Enum

class ContentEncoding(Enum):
    Zstd = 'zstd',
    Lz4 = 'lz4',
    Zlib = 'zlib',
    Null = 'null'

class ConfigOptions:
    queue_name: str
    rpc_exchange_name: str
    rpc_queue_name: str
    def __init__(self, queue_name: str, rpc_exchange_name: str, rpc_queue_name: str) -> None: ...

class TlsAdaptor:
    @staticmethod
    def with_client_auth(ca_path: Optional[str], cert_path: str, key_path: str, domain: str) -> "TlsAdaptor": ...
    @staticmethod
    def without_client_auth(root_ca_cert: Optional[str], domain: str) -> "TlsAdaptor": ...

class Config:
    host: str
    port: int
    username: str
    password: str
    virtual_host: str
    options: ConfigOptions

    def __init__(
        self,
        host: str,
        port: int,
        username: str,
        password: str,
        virtual_host: str,
        options: ConfigOptions
    ) -> None: ...


class QoSConfig:
    pub_confirm: bool
    rpc_client_confirm: bool
    rpc_server_confirm: bool
    sub_auto_ack: bool
    rpc_server_auto_ack: bool
    rpc_client_auto_ack: bool
    sub_prefetch: Optional[int]
    rpc_server_prefetch: Optional[int]
    rpc_client_prefetch: Optional[int]

    def __init__(self, pub_confirm: bool = True, rpc_client_confirm: bool = True, rpc_server_confirm: bool = False, sub_auto_ack: bool = False, rpc_server_auto_ack: bool = False, rpc_client_auto_ack: bool = False, sub_prefetch: Optional[int] = None, rpc_server_prefetch: Optional[int] = None, rpc_client_prefetch: Optional[int] = None) -> None:
        ...
    
    def default() -> 'QoSConfig':
        ...
    

class AsyncEventbus:
    def __init__(self, config: Config, qos_config: QoSConfig) -> None:
        ...

    async def publish(
        self, 
        exchange_name: str,
        routing_key: str,
        body: Union[bytes, str],
        content_type: Optional[str],
        content_encoding: ContentEncoding,
        command_timeout: Optional[int]
    ) -> None :
        """
        Sends a publish message to the bus following parameters passed

        Args:
            exchange: exchange name
            routing_key:  routing key name
            body: body that will be sent
            content_type: content type of message
            timeout: timeout in seconds for waiting for response
            connection_timeout: timeout for waiting for connection restabilishment
            delivery_mode: delivery mode
            expiration: maximum lifetime of message to stay on the queue

        Returns:
            None: if publish confirmation is setted to False
            True: if successful when publish confirmation is setted to True

        Raises:
            AutoReconnectException: when cannout reconnect on the gived timeout
            PublishTimeoutException: if publish confirmation is setted to True and \
            does not receive confirmation on the gived timeout
            NackException: if publish confirmation is setted to True and receives a nack


        Examples:
            >>> from json import dumps
            >>> exchange_name = "example.rpc"
            >>> routing_key = "user.find3"
            >>> await eventbus.publish(exchange_name, routing_key, dumps(["content_message"]), "application/json", ContentEncoding.Null, None)
        """
        ...

    async def rpc_client(
        self, 
        exchange_name: str,
        routing_key: str,
        body: Union[bytes, str],
        content_type: str,
        content_encoding: ContentEncoding,
        response_timeout: int,
        command_timeout: Optional[int],
        expiration: Optional[int],
    ) -> bytes:
        """
        Sends a publish message to queue of the bus and waits for a response

        Args:
            exchange: exchange name
            routing_key:  routing key name
            body: body that will be sent
            content_type: content type of message
            response_timeout: timeout in seconds for waiting for response
            command_timeout: timeout for waiting for command execution
            expiration: maximum lifetime of message to stay on the queue

        Returns:
            bytes: response message

        Raises:
            AutoReconnectException: when cannout reconnect on the gived timeout
            PublishTimeoutException: if publish confirmation is setted to True and \
            does not receive confirmation on the gived timeout
            NackException: if publish confirmation is setted to True and receives a nack
            ResponseTimeoutException: if response timeout is reached
            RpcProviderException: if the rpc provider responded with an error

        Examples:
            >>> from json import dumps
            >>> await eventbus.rpc_client("example.rpc", "user.find", dumps([{"name": "example"}]), "application/json")
        """
        ...

    async def subscribe(
        self,
        exchange_name: str,
        routing_key: str,
        handler: Callable[[List[bytes]], None],
        process_timeout: Optional[int],
        command_timeout: Optional[int],
    ) -> Any:
        ...


    async def rpc_server(
        self,
        routing_key: str,
        handler: Callable[[List[bytes]], Awaitable[List[bytes]]],
        process_timeout: Optional[int],
        command_timeout: Optional[int],
    ) -> Any:
        ...
        
    async def dispose(self) -> None: ...

class Payload:
    def __init__(self, data: bytes) -> None: ...