use std::{fs::File, io::BufReader, path::Path, sync::Arc};

use amqp_client_rust::{
    amqprs::{tls::TlsAdaptor as RuTlsAdaptor}, api::{
        eventbus::AsyncEventbusRabbitMQ as RuAsyncEventbusRabbitMQ,
        utils::{ContentEncoding as RuContentEncoding, DeliveryMode as RuDeliveryMode},
    }, domain::config::{
        Config as RuConfig, ConfigOptions as RuConfigOptions, QoSConfig as RuQoSConfig,
    }
};
use pyo3::{
    exceptions::PyValueError, prelude::*, types::{PyBytes, PyString}
};
pub mod exceptions;
use exceptions::AppError;
use rustls::{ClientConfig, RootCertStore, pki_types::{CertificateDer, PrivateKeyDer}};
use tokio_rustls::TlsConnector;
use std::path::PathBuf;

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
#[derive(Debug, Clone)]
pub enum DeliveryMode {
    Transient = 1,
    Persistent = 2,
}
impl From<DeliveryMode> for RuDeliveryMode {
    fn from(mode: DeliveryMode) -> Self {
        match mode {
            DeliveryMode::Transient => RuDeliveryMode::Transient,
            DeliveryMode::Persistent => RuDeliveryMode::Persistent,
        }
    }
}

#[pyclass(from_py_object, get_all, set_all)]
#[derive(Debug, Clone)]
pub enum ContentEncoding {
    Zstd,
    Lz4,
    Zlib,
    Null,
}
impl Into<RuContentEncoding> for ContentEncoding {
    fn into(self) -> RuContentEncoding {
        match self {
            ContentEncoding::Zstd => RuContentEncoding::Zstd,
            ContentEncoding::Lz4 => RuContentEncoding::Lz4,
            ContentEncoding::Zlib => RuContentEncoding::Zlib,
            ContentEncoding::Null => RuContentEncoding::None,
        }
    }
}

#[pyclass(from_py_object, get_all, set_all)]
#[derive(Debug, Clone)]
pub struct ConfigOptions {
    queue_name: String,
    rpc_exchange_name: String,
    rpc_queue_name: String,
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
            rpc_queue_name: options.rpc_queue_name,
        }
    }
}

#[pyclass(from_py_object, get_all, set_all)]
#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub virtual_host: String,
    pub options: ConfigOptions,
    pub tls_adaptor: Option<TlsAdaptor>, // Placeholder for TLS adaptor
}
#[pymethods]
impl Config {
    #[new]
    #[pyo3(signature = (host, port, username, password, virtual_host, options, tls_adaptor=None))]
    fn new(
        host: String,
        port: u16,
        username: String,
        password: String,
        virtual_host: String,
        options: ConfigOptions,
        tls_adaptor: Option<TlsAdaptor>,
    ) -> Self {
        Self {
            host,
            port,
            username,
            password,
            virtual_host,
            options,
            tls_adaptor,
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
            options: config.options.into(),
            tls_adaptor: config.tls_adaptor.map(|t| t.into()),
        }
    }
}
#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct TlsAdaptor {
    pub(crate) inner: Arc<RuTlsAdaptor>,
}

impl TlsAdaptor {
    fn build_root_store(root_ca_cert: Option<&Path>) -> std::io::Result<RootCertStore> {
        let mut root_store = RootCertStore::empty();
        if let Some(root_ca_cert) = root_ca_cert {
            let mut pem = BufReader::new(File::open(root_ca_cert)?);

            let certs = rustls_pemfile::certs(&mut pem);

            let trust_anchors = certs
                .into_iter()
                .map(|cert| {
                    cert.map(|cert| {
                        let anchor = webpki::anchor_from_trusted_cert(&cert).unwrap().to_owned();

                        rustls_pki_types::TrustAnchor {
                            subject: anchor.subject,
                            subject_public_key_info: anchor.subject_public_key_info,
                            name_constraints: anchor.name_constraints,
                        }
                    })
                })
                .collect::<std::io::Result<Vec<rustls_pki_types::TrustAnchor>>>()?;

            root_store.roots.extend(trust_anchors);
        } else {
            root_store
                .roots
                .extend(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
                    rustls_pki_types::TrustAnchor {
                        subject: ta.subject.clone(),
                        subject_public_key_info: ta.subject_public_key_info.clone(),
                        name_constraints: ta.name_constraints.clone(),
                    }
                }));
        }
        Ok(root_store)
    }

    fn build_client_certificates<'a>(
        client_cert: &Path,
    ) -> std::io::Result<Vec<CertificateDer<'a>>> {
        let file = File::open(client_cert)?;
        let mut pem = BufReader::new(file);
        let raw_certs = rustls_pemfile::certs(&mut pem);

        let certs: Vec<CertificateDer> = raw_certs
            .into_iter()
            .collect::<std::io::Result<Vec<CertificateDer>>>()?;
        Ok(certs)
    }

    fn build_client_private_keys<'a>(
        client_private_key: &Path,
    ) -> std::io::Result<Vec<PrivateKeyDer<'a>>> {
        let mut pem = BufReader::new(File::open(client_private_key)?);
        let keys = TlsAdaptor::read_private_keys_from_pem(&mut pem)?;
        let keys = keys
            .into_iter()
            .map(|c| {
                PrivateKeyDer::try_from(c)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            })
            .collect::<std::io::Result<Vec<PrivateKeyDer>>>()?;

        Ok(keys)
    }
    fn read_private_keys_from_pem(
        rd: &mut dyn std::io::BufRead,
    ) -> Result<Vec<Vec<u8>>, std::io::Error> {
        let mut keys = Vec::new();

        loop {
            match rustls_pemfile::read_one(rd)? {
                None => return Ok(keys),
                Some(rustls_pemfile::Item::Pkcs1Key(key)) => {
                    keys.push(key.secret_pkcs1_der().to_vec())
                } //PKCS1/RSA
                Some(rustls_pemfile::Item::Pkcs8Key(key)) => {
                    keys.push(key.secret_pkcs8_der().to_vec())
                } //PKCS8
                Some(rustls_pemfile::Item::Sec1Key(key)) => {
                    keys.push(key.secret_sec1_der().to_vec())
                } //SEC1/EC
                _ => {}
            };
        }
    }
}

fn install_crypto_provider() -> Result<(), PyErr> {
    #[cfg(not(target_os = "linux"))]
    rustls::crypto::ring::default_provider()
    .install_default().map_err(|_| PyValueError::new_err("Error on install crypto provider for tls"))?;
    #[cfg(target_os = "linux")]
    rustls::crypto::aws_lc_rs::default_provider()
    .install_default().map_err(|_| PyValueError::new_err("Error on install crypto provider for tls"))?;
    Ok(())
}

#[pymethods]
impl TlsAdaptor {
    #[staticmethod]
    pub fn with_client_auth(
        ca_path: Option<PathBuf>,
        cert_path: PathBuf,
        key_path: PathBuf,
        domain: String,
    ) -> PyResult<Self> {
        install_crypto_provider()?;
        let root_cert_store: RootCertStore = TlsAdaptor::build_root_store(ca_path.as_deref())?;
        let client_certs: Vec<CertificateDer> = TlsAdaptor::build_client_certificates(&cert_path)?;
        let client_keys: Vec<PrivateKeyDer> = TlsAdaptor::build_client_private_keys(&key_path)?;
        let config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_client_auth_cert(client_certs, client_keys.into_iter().next().ok_or_else(|| PyValueError::new_err("No valid private keys found in the provided key file"))?)
            .unwrap();
        let connector = TlsConnector::from(Arc::new(config));

        let inner = Arc::new(
            RuTlsAdaptor::new(connector, domain)
        );

        Ok(Self { inner })
    }
    #[staticmethod]
    pub fn without_client_auth(root_ca_cert: Option<PathBuf>, domain: String) -> PyResult<Self> {
        install_crypto_provider()?;
        let inner = Arc::new(
            RuTlsAdaptor::without_client_auth(root_ca_cert.as_deref(), domain)
                .map_err(|e| PyValueError::new_err(e.to_string()))?,
        );
        Ok(Self { inner })
    }
}
impl From<TlsAdaptor> for RuTlsAdaptor {
    fn from(adaptor: TlsAdaptor) -> Self {
        Arc::unwrap_or_clone(adaptor.inner)
    }
}

#[pyclass(from_py_object, get_all, set_all)]
#[derive(Debug, Clone)]
pub struct QoSConfig {
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
    fn new(
        pub_confirm: bool,
        rpc_client_confirm: bool,
        rpc_server_confirm: bool,
        sub_auto_ack: bool,
        rpc_server_auto_ack: bool,
        rpc_client_auto_ack: bool,
        sub_prefetch: Option<u16>,
        rpc_server_prefetch: Option<u16>,
        rpc_client_prefetch: Option<u16>,
    ) -> Self {
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
            rpc_client_prefetch: None,
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
            rpc_client_prefetch: config.rpc_client_prefetch,
        }
    }
}

#[derive(FromPyObject)]
pub enum Payload<'py> {
    Bytes(Bound<'py, PyBytes>),
    Str(Bound<'py, PyString>),
}
#[pymethods]
impl AsyncEventbus {
    #[new]
    fn new(config: Config, qos_config: QoSConfig) -> Self {
        let rt = pyo3_async_runtimes::tokio::get_runtime();

        let _guard = rt.enter();
        Self {
            eventbus: Arc::new(RuAsyncEventbusRabbitMQ::new(
                config.into(),
                qos_config.into(),
            )),
        }
    }

    #[pyo3(signature = (exchange_name, routing_key, body, content_type=Some("application/json"), content_encoding=ContentEncoding::Null, publish_timeout=16, connection_timeout=16, delivery_mode=DeliveryMode::Transient, expiration=None))]
    fn publish<'py>(
        slf: PyRef<'py, Self>,
        exchange_name: &'py str,
        routing_key: &'py str,
        body: Payload,
        content_type: Option<&'py str>,
        content_encoding: ContentEncoding,
        publish_timeout: Option<u64>,
        connection_timeout: Option<u64>,
        delivery_mode: DeliveryMode,
        expiration: Option<u32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let eventbus = Arc::clone(&slf.eventbus);
        let py = slf.py();

        let exchange_name = exchange_name.to_owned();
        let routing_key = routing_key.to_owned();
        let payload_bytes = match body {
            Payload::Bytes(b) => b.as_bytes().to_vec(),
            Payload::Str(s) => s.to_str()?.as_bytes().to_vec(),
        };

        let content_type = content_type.map(|s| s.to_owned());
        let content_encoding = content_encoding.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let publish_timeout = publish_timeout.map(std::time::Duration::from_secs);
            let connection_timeout = connection_timeout.map(std::time::Duration::from_secs);
            match eventbus
                .publish(
                    &exchange_name,
                    &routing_key,
                    payload_bytes,
                    content_type.as_deref(),
                    content_encoding.into(),
                    publish_timeout,
                    connection_timeout,
                    Some(delivery_mode.into()),
                    expiration,
                )
                .await
            {
                Ok(res) => Ok(res),
                Err(e) => return Err(AppError::from(e).into()),
            }
        })
    }

    #[pyo3(signature = (exchange_name, routing_key, body, content_type="application/json", content_encoding=ContentEncoding::Null, response_timeout=20_000, connection_timeout=Some(32), delivery_mode=DeliveryMode::Transient, expiration=None))]
    fn rpc_client<'py>(
        slf: PyRef<'py, Self>,
        exchange_name: &str,
        routing_key: &str,
        body: Payload<'py>,
        content_type: &str,
        content_encoding: ContentEncoding,
        response_timeout: u32,
        connection_timeout: Option<u64>,
        delivery_mode: DeliveryMode,
        expiration: Option<u32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let eventbus = Arc::clone(&slf.eventbus);

        let exchange_name = exchange_name.to_owned();
        let routing_key = routing_key.to_owned();
        let payload_bytes = match body {
            Payload::Bytes(b) => b.as_bytes().to_vec(),
            Payload::Str(s) => s.to_str()?.as_bytes().to_vec(),
        };
        let content_type = content_type.to_owned();
        let content_encoding = content_encoding.clone();

        pyo3_async_runtimes::tokio::future_into_py(slf.py(), async move {
            let conn_timeout = connection_timeout.map(std::time::Duration::from_secs);
            let response = match eventbus
                .rpc_client(
                    &exchange_name,
                    &routing_key,
                    payload_bytes,
                    &content_type,
                    content_encoding.into(),
                    response_timeout,
                    conn_timeout,
                    Some(delivery_mode.into()),
                    expiration,
                )
                .await
            {
                Ok(res) => Ok(res),
                Err(e) => Err(AppError::from(e).into()),
            };
            response
        })
    }

    #[pyo3(signature = (exchange_name, routing_key, handler, process_timeout=None, command_timeout=Some(16)))]
    fn subscribe<'py>(
        slf: PyRef<'py, Self>,
        exchange_name: &str,
        routing_key: &str,
        handler: Py<PyAny>,
        process_timeout: Option<u64>,
        command_timeout: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let eventbus = Arc::clone(&slf.eventbus);
        let locals = pyo3_async_runtimes::TaskLocals::with_running_loop(slf.py())?;
        let py = slf.py();
        let handler = Arc::new(handler);
        let exchange_name = exchange_name.to_owned();
        let routing_key = routing_key.to_owned();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let process_timeout = process_timeout.map(std::time::Duration::from_secs);
            let command_timeout = command_timeout.map(std::time::Duration::from_secs);

            match eventbus
                .subscribe(
                    &exchange_name,
                    &routing_key,
                    move |body| {
                        let handler_clone = handler.clone();
                        let locals_clone = locals.clone();
                        async move {
                            pyo3_async_runtimes::tokio::scope(locals_clone, async move {
                                let future_result = Python::attach(|py| -> PyResult<_> {
                                    let bound_handler = handler_clone.bind(py);
                                    let coro = bound_handler.call1((body,))?;

                                    pyo3_async_runtimes::tokio::into_future(coro)
                                });
                                match future_result {
                                    Ok(py_future) => match py_future.await {
                                        Ok(_) => Ok(()),
                                        Err(e) => Err(Box::new(std::io::Error::new(
                                            std::io::ErrorKind::Other,
                                            e.to_string(),
                                        ))
                                            as Box<dyn std::error::Error + Send + Sync>),
                                    },
                                    Err(e) => Err(Box::new(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        format!("Failed to execute Python callback: {}", e),
                                    ))
                                        as Box<dyn std::error::Error + Send + Sync>),
                                }
                            })
                            .await
                        }
                    },
                    process_timeout,
                    command_timeout,
                )
                .await
            {
                Ok(res) => Ok(res),
                Err(e) => Err(AppError::from(e).into()),
            }
        })
    }

    #[pyo3(signature = (routing_key, handler, process_timeout=None, command_timeout=None))]
    fn provide_resource<'py>(
        slf: PyRef<'py, Self>,
        routing_key: &str,
        handler: Py<PyAny>,
        process_timeout: Option<u64>,
        command_timeout: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let eventbus = Arc::clone(&slf.eventbus);
        let locals = pyo3_async_runtimes::TaskLocals::with_running_loop(slf.py())?;
        let py = slf.py();
        let handler = Arc::new(handler);
        let routing_key = routing_key.to_owned();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let process_timeout = process_timeout.map(std::time::Duration::from_secs);
            let command_timeout = command_timeout.map(std::time::Duration::from_secs);

            match eventbus
                .provide_resource(
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
                                });
                                match py_future_result {
                                    Ok(py_future) => match py_future.await {
                                        Ok(result) => Python::attach(|py| {
                                            match result.cast_bound::<PyBytes>(py) {
                                                Ok(bytes) => Ok(bytes.as_bytes().to_vec()),
                                                Err(_) => Err(Box::new(std::io::Error::new(
                                                    std::io::ErrorKind::InvalidData,
                                                    "RPC handler must return bytes",
                                                ))
                                                    as Box<dyn std::error::Error + Send + Sync>),
                                            }
                                        }),
                                        Err(e) => Err(Box::new(std::io::Error::new(
                                            std::io::ErrorKind::Other,
                                            e.to_string(),
                                        ))
                                            as Box<dyn std::error::Error + Send + Sync>),
                                    },
                                    Err(e) => Err(Box::new(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        format!("Failed to execute Python callback: {}", e),
                                    ))
                                        as Box<dyn std::error::Error + Send + Sync>),
                                }
                            })
                            .await
                        }
                    },
                    process_timeout,
                    command_timeout,
                )
                .await
            {
                Ok(res) => Ok(res),
                Err(e) => Err(AppError::from(e).into()),
            }
        })
    }
    fn dispose(slf: PyRef<'_, Self>) -> PyResult<Bound<'_, PyAny>> {
        let eventbus = Arc::clone(&slf.eventbus); // Clone the Arc for the async move
        let py = slf.py();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            eventbus.dispose().await.map_err(|_| {
                AppError {
                    description: None,
                    message: None,
                    error_type: amqp_client_rust::errors::AppErrorType::UnexpectedResultError,
                }
                .into()
            })
        })
    }
}

#[pymodule]
fn amqp_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<AsyncEventbus>()?;
    m.add_class::<Config>()?;
    m.add_class::<ConfigOptions>()?;
    m.add_class::<QoSConfig>()?;
    m.add_class::<TlsAdaptor>()?;
    m.add_class::<ContentEncoding>()?;
    Ok(())
}
