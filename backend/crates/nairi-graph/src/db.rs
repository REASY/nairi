use rsmgclient::{ConnectParams, Connection, ConnectionStatus, QueryParam, SSLMode, TrustCallback};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum MemgraphError {
    #[error("ConnectionError: {0}")]
    ConnectionError(String),
    #[error("QueryError: {0}")]
    QueryError(String),
    #[error("CommitError: {0}")]
    CommitError(String),
}

pub struct Memgraph {
    connection: Connection,
    connect_params: ConnectParamsSnapshot,
}

struct ConnectParamsSnapshot {
    port: u16,
    host: Option<String>,
    address: Option<String>,
    username: Option<String>,
    password: Option<String>,
    client_name: String,
    sslmode: SSLMode,
    sslcert: Option<String>,
    sslkey: Option<String>,
    trust_callback: Option<TrustCallback>,
    lazy: bool,
    autocommit: bool,
}

fn clone_sslmode(mode: &SSLMode) -> SSLMode {
    match mode {
        SSLMode::Disable => SSLMode::Disable,
        SSLMode::Require => SSLMode::Require,
    }
}

impl ConnectParamsSnapshot {
    fn from_params(params: &ConnectParams) -> Self {
        Self {
            port: params.port,
            host: params.host.clone(),
            address: params.address.clone(),
            username: params.username.clone(),
            password: params.password.clone(),
            client_name: params.client_name.clone(),
            sslmode: clone_sslmode(&params.sslmode),
            sslcert: params.sslcert.clone(),
            sslkey: params.sslkey.clone(),
            trust_callback: params.trust_callback,
            lazy: params.lazy,
            autocommit: params.autocommit,
        }
    }

    fn to_params(&self) -> ConnectParams {
        ConnectParams {
            port: self.port,
            host: self.host.clone(),
            address: self.address.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            client_name: self.client_name.clone(),
            sslmode: clone_sslmode(&self.sslmode),
            sslcert: self.sslcert.clone(),
            sslkey: self.sslkey.clone(),
            trust_callback: self.trust_callback,
            lazy: self.lazy,
            autocommit: self.autocommit,
        }
    }
}

pub struct QuerySpec {
    pub query: String,
    pub params: HashMap<String, QueryParam>,
}

impl QuerySpec {
    #[allow(dead_code)]
    pub fn new(query: String) -> Self {
        Self {
            query,
            params: HashMap::new(),
        }
    }

    pub fn with_params(query: String, params: HashMap<String, QueryParam>) -> Self {
        Self { query, params }
    }

    pub fn params(&self) -> Option<&HashMap<String, QueryParam>> {
        if self.params.is_empty() {
            None
        } else {
            Some(&self.params)
        }
    }
}

impl Memgraph {
    pub fn try_new(params: &ConnectParams) -> Result<Self, MemgraphError> {
        let connect_params = ConnectParamsSnapshot::from_params(params);
        let connection: Connection = Connection::connect(params)
            .map_err(|e| MemgraphError::ConnectionError(e.to_string()))?;
        let status = connection.status();
        if status != ConnectionStatus::Ready {
            return Err(MemgraphError::ConnectionError(format!(
                "Connection status {status:?}"
            )));
        }

        Ok(Self {
            connection,
            connect_params,
        })
    }

    fn ensure_connected(&mut self) -> Result<(), MemgraphError> {
        let status = self.connection.status();
        if status == ConnectionStatus::Bad || status == ConnectionStatus::Closed {
            self.reconnect()?;
        }
        Ok(())
    }

    fn reconnect(&mut self) -> Result<(), MemgraphError> {
        let params = self.connect_params.to_params();
        let connection: Connection = Connection::connect(&params)
            .map_err(|e| MemgraphError::ConnectionError(e.to_string()))?;
        let status = connection.status();
        if status != ConnectionStatus::Ready {
            return Err(MemgraphError::ConnectionError(format!(
                "Connection status {status:?}"
            )));
        }
        self.connection = connection;
        Ok(())
    }

    pub fn reconnect_if_bad(&mut self) {
        let status = self.connection.status();
        if status == ConnectionStatus::Bad || status == ConnectionStatus::Closed {
            if let Err(err) = self.reconnect() {
                eprintln!("Failed to reconnect memgraph after bad connection: {err}");
            }
        }
    }

    pub fn execute_query_spec(&mut self, spec: &QuerySpec) -> Result<(), MemgraphError> {
        self.ensure_connected()?;

        let cols = self.connection.execute(&spec.query, spec.params());
        match cols {
            Ok(_) => {}
            Err(err) => {
                let msg = err.to_string();
                self.reconnect_if_bad();
                return Err(MemgraphError::QueryError(msg).into());
            }
        };

        if let Err(e) = self.connection.fetchall() {
            let msg = e.to_string();
            self.reconnect_if_bad();
            return Err(MemgraphError::QueryError(msg).into());
        }

        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), MemgraphError> {
        self.connection.commit().map_err(|e| {
            let msg = e.to_string();
            self.reconnect_if_bad();
            MemgraphError::CommitError(msg)
        })
    }
}
