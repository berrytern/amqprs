from typing import Callable, Optional, Any, List, Awaitable
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
        body: bytes,
        content_type: Optional[str],
        content_encoding: ContentEncoding,
        command_timeout: Optional[int]
    ) -> None :
        ...
    async def rpc_client(
        self, 
        exchange_name: str,
        routing_key: str,
        body: bytes,
        content_type: str,
        content_encoding: ContentEncoding,
        timeout_millis: int,
        command_timeout: Optional[int],
        expiration: Optional[int],
    ) -> Any:
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