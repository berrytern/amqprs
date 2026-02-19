use std::sync::Arc;

use pyo3::{prelude::*, types::PyBytes};
use amqp_client_rust::{
    api::eventbus::AsyncEventbusRabbitMQ as RuAsyncEventbusRabbitMQ,
    domain::config::{Config as RuConfig, ConfigOptions as RuConfigOptions, QoSConfig as RuQoSConfig},
};
pub mod exceptions;
use exceptions::AppError;
/*static TOKIO_RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});*/
#[pyclass(skip_from_py_object)]
#[derive(Clone)]
struct AsyncEventbus {
    eventbus: Arc<RuAsyncEventbusRabbitMQ>,
}


#[pyclass(from_py_object, get_all, set_all)]
#[derive(Debug,Clone)]
pub struct ConfigOptions{
    queue_name: String,
    rpc_exchange_name: String,
    rpc_queue_name: String
}
#[pymethods]
impl ConfigOptions {
    #[new]
    fn new(queue_name: String, rpc_exchange_name: String, rpc_queue_name: String) -> Self {
        Self {
            queue_name,
            rpc_exchange_name,
            rpc_queue_name,
        }
    }
}
impl From<ConfigOptions> for RuConfigOptions {
    fn from(options: ConfigOptions) -> Self {
        Self {
            queue_name: options.queue_name,
            rpc_exchange_name: options.rpc_exchange_name,
            rpc_queue_name: options.rpc_queue_name
        }
    }
}

#[pyclass(from_py_object, get_all, set_all)]
#[derive(Debug,Clone)]
pub struct Config{
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub virtual_host: String,
    pub options: ConfigOptions
}
#[pymethods]
impl Config {
    #[new]
    fn new(host: String, port: u16, username: String, password: String, virtual_host: String, options: ConfigOptions) -> Self {
        Self {
            host,
            port,
            username,
            password,
            virtual_host,
            options,
        }
    }
}

impl From<Config> for RuConfig {
    fn from(config: Config) -> Self {
        Self {
            host: config.host,
            port: config.port,
            username: config.username,
            password: config.password,
            virtual_host: config.virtual_host,
            options: config.options.into()
        }
    }
}

#[pyclass(from_py_object, get_all, set_all)]
#[derive(Debug,Clone)]
pub struct QoSConfig{
    pub pub_confirm: bool,
    pub rpc_client_confirm: bool,
    pub rpc_server_confirm: bool,
    pub sub_auto_ack: bool,
    pub rpc_server_auto_ack: bool,
    pub rpc_client_auto_ack: bool,
    pub sub_prefetch: Option<u16>,
    pub rpc_server_prefetch: Option<u16>,
    pub rpc_client_prefetch: Option<u16>,
}
#[pymethods]
impl QoSConfig {
    #[new]
    #[pyo3(signature = (pub_confirm=true, rpc_client_confirm=true, rpc_server_confirm=false, sub_auto_ack=false, rpc_server_auto_ack=false, rpc_client_auto_ack=false, sub_prefetch=None, rpc_server_prefetch=None, rpc_client_prefetch=None))]
    fn new(pub_confirm: bool, rpc_client_confirm: bool, rpc_server_confirm: bool, sub_auto_ack: bool, rpc_server_auto_ack: bool, rpc_client_auto_ack: bool, sub_prefetch: Option<u16>, rpc_server_prefetch: Option<u16>, rpc_client_prefetch: Option<u16>) -> Self {
        Self {
            pub_confirm,
            rpc_client_confirm,
            rpc_server_confirm,
            sub_auto_ack,
            rpc_server_auto_ack,
            rpc_client_auto_ack,
            sub_prefetch,
            rpc_server_prefetch,
            rpc_client_prefetch,
        }
    }

    #[staticmethod]
    pub fn default() -> Self {
        Self {
            pub_confirm: true,
            rpc_client_confirm: true,
            rpc_server_confirm: false,
            sub_auto_ack: false,
            rpc_server_auto_ack: false,
            rpc_client_auto_ack: false,
            sub_prefetch: None,
            rpc_server_prefetch: None,
            rpc_client_prefetch: None
        }
    }
}
impl From<QoSConfig> for RuQoSConfig {
    fn from(config: QoSConfig) -> Self {
        Self {
            pub_confirm: config.pub_confirm,
            rpc_client_confirm: config.rpc_client_confirm,
            rpc_server_confirm: config.rpc_server_confirm,
            sub_auto_ack: config.sub_auto_ack,
            rpc_server_auto_ack: config.rpc_server_auto_ack,
            rpc_client_auto_ack: config.rpc_client_auto_ack,
            sub_prefetch: config.sub_prefetch,
            rpc_server_prefetch: config.rpc_server_prefetch,
            rpc_client_prefetch: config.rpc_client_prefetch
        }
    }
}

#[pymethods]
impl AsyncEventbus {
    #[new]
    fn new(config: Config, qos_config: QoSConfig) -> Self {

        let rt = pyo3_async_runtimes::tokio::get_runtime();

        let _guard = rt.enter();
        Self {
            eventbus: Arc::new(RuAsyncEventbusRabbitMQ::new(config.into(), qos_config.into()))
        }
    }

    fn publish(
        slf: PyRef<'_, Self>, 
        exchange_name: String,
        routing_key: String,
        body: Vec<u8>,
        content_type: Option<String>,
        command_timeout: Option<u64>
    ) -> PyResult<Bound<'_, PyAny>> {
        let eventbus = Arc::clone(&slf.eventbus); // Clone the Arc for the async move
        let py = slf.py();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let command_timeout = command_timeout.map(std::time::Duration::from_secs);
            match eventbus.publish(
                &exchange_name, 
                &routing_key, 
                body, 
                content_type.as_deref(),
                command_timeout
            ).await {
                Ok(res) => Ok(res),
                Err(e) => return Err(AppError::from(e).into()),
            }
        })
    }
    fn rpc_client(
        slf: PyRef<'_, Self>,
        exchange_name: String,
        routing_key: String,
        body: Vec<u8>,
        content_type: String,
        timeout_millis: u32,
        connection_timeout: Option<u64>,
        expiration: Option<u32>,
    ) -> PyResult<Bound<'_, PyAny>> {
        //let rt = pyo3_async_runtimes::tokio::get_runtime();

        //let _guard = rt.enter();
        let eventbus = Arc::clone(&slf.eventbus);
        pyo3_async_runtimes::tokio::future_into_py(slf.py(), async move {
            let conn_timeout = connection_timeout.map(std::time::Duration::from_secs);
            let response = match eventbus
                .rpc_client(&exchange_name, &routing_key, body, &content_type, timeout_millis, conn_timeout, expiration)
                .await {
                Ok(res) => Ok(res),
                Err(e) => Err(AppError::from(e).into()),
            };
            response
        })
    }

    /// Subscribe to a queue with a Python callback
    fn subscribe(
        slf: PyRef<'_, Self>,
        exchange_name: String,
        routing_key: String,
        handler: Py<PyAny>,
        process_timeout: Option<u64>,
        command_timeout: Option<u64>,
    ) -> PyResult<Bound<'_, PyAny>> {
        let rt = pyo3_async_runtimes::tokio::get_runtime();

        let _guard = rt.enter();
        let eventbus = Arc::clone(&slf.eventbus);
        let locals = pyo3_async_runtimes::TaskLocals::with_running_loop(slf.py())?;
        let py = slf.py();
        let handler = Arc::new(handler);

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let process_timeout = process_timeout.map(std::time::Duration::from_secs);
            let command_timeout = command_timeout.map(std::time::Duration::from_secs);
            
            match eventbus.subscribe(
                &exchange_name,
                &routing_key,
                move |body| {
                    let handler_clone = handler.clone();
                    let locals_clone = locals.clone();
                    async move {
                        pyo3_async_runtimes::tokio::scope(locals_clone, async move {
                            
                            let py_future_result = Python::attach(|py| -> PyResult<_> {
                                let bound_handler = handler_clone.bind(py);
                                let coro = bound_handler.call1((body,))?;
                                
                                pyo3_async_runtimes::tokio::into_future(coro)
                            });
                            match py_future_result {
                                Ok(py_future) => {
                                    match py_future.await {
                                        Ok(_) => Ok(()),
                                        Err(e) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn std::error::Error + Send + Sync>)
                                    }
                                },
                                Err(e) => {
                                    Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to execute Python callback: {}", e))) as Box<dyn std::error::Error + Send + Sync>)
                                }
                            }
                        }).await
                    }
                },
                process_timeout,
                command_timeout,
            ).await {
                Ok(res) => Ok(res),
                Err(e) => Err(AppError::from(e).into()),
            }
        })
    }

    fn rpc_server(
        slf: PyRef<'_, Self>,
        routing_key: String,
        handler: Py<PyAny>,
        process_timeout: Option<u64>,
        command_timeout: Option<u64>,
    ) -> PyResult<Bound<'_, PyAny>> {
        let eventbus = Arc::clone(&slf.eventbus);
        let locals = pyo3_async_runtimes::TaskLocals::with_running_loop(slf.py())?;
        let py = slf.py();
        let handler = Arc::new(handler);

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let process_timeout = process_timeout.map(std::time::Duration::from_secs);
            let command_timeout = command_timeout.map(std::time::Duration::from_secs);

            match eventbus.provide_resource(
                &routing_key,
                move |body| {
                    let handler_clone = handler.clone(); 
                    let locals_clone = locals.clone();
                    async move {
                        pyo3_async_runtimes::tokio::scope(locals_clone, async move {
                            
                            let py_future_result = Python::attach(|py| -> PyResult<_> {
                                let bound_handler = handler_clone.bind(py);
                                let coro = bound_handler.call1((body,))?;
                                
                                // Now into_future will successfully find the asyncio loop!
                                pyo3_async_runtimes::tokio::into_future(coro)
                            });match py_future_result {
                                Ok(py_future) => {
                                    match py_future.await {
                                        Ok(result) => {
                                            Python::attach(|py| {
                                                match result.cast_bound::<PyBytes>(py) {
                                                    Ok(bytes) => Ok(bytes.as_bytes().to_vec()),
                                                    Err(_) => Err(Box::new(std::io::Error::new(
                                                        std::io::ErrorKind::InvalidData, 
                                                        "RPC handler must return bytes"
                                                    )) as Box<dyn std::error::Error + Send + Sync>)
                                                }
                                            })
                                        },
                                        Err(e) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn std::error::Error + Send + Sync>)
                                    }
                                },
                                Err(e) => {
                                    Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to execute Python callback: {}", e))) as Box<dyn std::error::Error + Send + Sync>)
                                }
                            }
                        }).await // <-- Don't forget to await the scope!
                    }
                },
                process_timeout,
                command_timeout
            ).await {
                Ok(res) => Ok(res),
                Err(e) => Err(AppError::from(e).into()),
            }
        })
    }
    fn dispose(slf: PyRef<'_, Self>) -> PyResult<Bound<'_, PyAny>> {
        let eventbus = Arc::clone(&slf.eventbus); // Clone the Arc for the async move
        let py = slf.py();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            eventbus.dispose(
            ).await.map_err(|_| AppError{description:None, message: None, error_type: amqp_client_rust::errors::AppErrorType::UnexpectedResultError }.into())
        })

    }
}


#[pymodule]
fn amqp_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<AsyncEventbus>()?;
    m.add_class::<Config>()?;
    m.add_class::<ConfigOptions>()?;
    m.add_class::<QoSConfig>()?;
    Ok(())
}
