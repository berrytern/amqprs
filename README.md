# amqp-rs

[![License](https://img.shields.io/badge/license-Apache%202-blue.svg)](LICENSE)
<a href="https://pypi.org/project/amqp-rs" target="_blank">
    <img src="https://img.shields.io/pypi/v/amqpr?color=%2334D058&label=pypi%20package" alt="Package version">
</a>
<a href="https://pypi.org/project/amqp-rs" target="_blank">
    <img src="https://img.shields.io/pypi/pyversions/amqpr.svg?color=%2334D058" alt="Supported Python versions">
</a>

**amqp-rs** is a Python client library with a high level of abstraction for manipulating messages in RabbitMQ.

---

### What is "amqprs"
A FFI of rust's library [amqp-client-python](https://github.com/berrytern/amqp-client-rust) that inherit performance and stability of it.

### Features
- Thread Safe;
- Asynchronous API for high performance;
- Automatic creation and management of queues, exchanges, and channels;
- Connection persistence and automatic reconnection;
- Support for **direct**, **topic**, and **fanout** exchanges;
- Message Publishing and Subscribing;
- Built-in support for Remote Procedure Calls (RPC).
- Encoding/Decoding: support to zstd, zlib and lz4;
- Gracefull shutdowns: stop consumers and process remaining messages;

---

### Installation

You can install `amqp-rs` directly from PyPI using uv:

```bash
uv add amqp-rs