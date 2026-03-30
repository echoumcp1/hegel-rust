use crate::cbor_utils::{cbor_map, map_insert};
use crate::protocol::{Connection, SERVER_CRASHED_MESSAGE, Stream};
use crate::runner::Verbosity;
use ciborium::Value;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, LazyLock};

/// Error indicating the server ran out of data for this test case.
#[derive(Debug)]
pub struct StopTestError;
impl std::fmt::Display for StopTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Server ran out of data (StopTest)")
    }
}
impl std::error::Error for StopTestError {}

/// Backend for test case data generation.
///
/// Abstracts all communication with a data source (e.g. the hegel-core server)
/// behind typed methods. Each fallible method returns `Result<T, StopTestError>`
/// for operations that can be cut short by data exhaustion.
///
/// All methods take `&self` — implementations use interior mutability as needed.
pub trait Backend {
    /// Send a CBOR schema and receive a generated CBOR value.
    fn generate(&self, schema: &Value) -> Result<Value, StopTestError>;

    /// Begin a labeled span (used for composite generator structure).
    fn start_span(&self, label: u64) -> Result<(), StopTestError>;

    /// End the current span. If `discard` is true, the span's choices are discarded.
    fn stop_span(&self, discard: bool) -> Result<(), StopTestError>;

    /// Create a new server-managed collection. Returns an opaque handle.
    fn new_collection(
        &self,
        name: &str,
        min_size: u64,
        max_size: Option<u64>,
    ) -> Result<String, StopTestError>;

    /// Ask whether the collection should produce another element.
    fn collection_more(&self, collection: &str) -> Result<bool, StopTestError>;

    /// Reject the last element drawn from a collection.
    fn collection_reject(&self, collection: &str, why: Option<&str>) -> Result<(), StopTestError>;

    /// Create a new variable pool. Returns an opaque pool id.
    fn new_pool(&self) -> Result<i128, StopTestError>;

    /// Register a new variable in the pool. Returns the variable id.
    fn pool_add(&self, pool_id: i128) -> Result<i128, StopTestError>;

    /// Draw a variable id from the pool.
    /// If `consume` is true, the variable is removed from the pool.
    fn pool_generate(&self, pool_id: i128, consume: bool) -> Result<i128, StopTestError>;

    /// Signal that the test case is complete.
    fn mark_complete(&self, status: &str, origin: Option<&str>);

    /// Returns true if a previous request triggered an abort (overflow/StopTest).
    fn test_aborted(&self) -> bool;
}

static PROTOCOL_DEBUG: LazyLock<bool> = LazyLock::new(|| {
    matches!(
        std::env::var("HEGEL_PROTOCOL_DEBUG")
            .unwrap_or_default()
            .to_lowercase()
            .as_str(),
        "1" | "true"
    )
});

/// Backend implementation that communicates with the hegel-core server
/// over a multiplexed stream.
pub(crate) struct ServerBackend {
    connection: Arc<Connection>,
    stream: RefCell<Stream>,
    aborted: Cell<bool>,
    verbosity: Verbosity,
}

impl ServerBackend {
    pub(crate) fn new(connection: Arc<Connection>, stream: Stream, verbosity: Verbosity) -> Self {
        ServerBackend {
            connection,
            stream: RefCell::new(stream),
            aborted: Cell::new(false),
            verbosity,
        }
    }

    fn send_request(&self, command: &str, payload: &Value) -> Result<Value, StopTestError> {
        if self.aborted.get() {
            return Err(StopTestError);
        }
        let debug = *PROTOCOL_DEBUG || self.verbosity == Verbosity::Debug;

        let mut entries = vec![(
            Value::Text("command".to_string()),
            Value::Text(command.to_string()),
        )];

        if let Value::Map(map) = payload {
            for (k, v) in map {
                entries.push((k.clone(), v.clone()));
            }
        }

        let request = Value::Map(entries);

        if debug {
            eprintln!("REQUEST: {:?}", request);
        }

        let result = self.stream.borrow_mut().request_cbor(&request);

        match result {
            Ok(response) => {
                if debug {
                    eprintln!("RESPONSE: {:?}", response);
                }
                Ok(response)
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("overflow")
                    || error_msg.contains("StopTest")
                    || error_msg.contains("stream is closed")
                {
                    if debug {
                        eprintln!("RESPONSE: StopTest/overflow");
                    }
                    self.stream.borrow_mut().mark_closed();
                    self.aborted.set(true);
                    Err(StopTestError)
                } else if error_msg.contains("FlakyStrategyDefinition")
                    || error_msg.contains("FlakyReplay")
                {
                    self.stream.borrow_mut().mark_closed();
                    self.aborted.set(true);
                    Err(StopTestError)
                } else if self.connection.server_has_exited() {
                    panic!("{}", SERVER_CRASHED_MESSAGE);
                } else {
                    panic!("Failed to communicate with Hegel: {}", e);
                }
            }
        }
    }
}

impl Backend for ServerBackend {
    fn generate(&self, schema: &Value) -> Result<Value, StopTestError> {
        self.send_request("generate", &cbor_map! {"schema" => schema.clone()})
    }

    fn start_span(&self, label: u64) -> Result<(), StopTestError> {
        self.send_request("start_span", &cbor_map! {"label" => label})?;
        Ok(())
    }

    fn stop_span(&self, discard: bool) -> Result<(), StopTestError> {
        self.send_request("stop_span", &cbor_map! {"discard" => discard})?;
        Ok(())
    }

    fn new_collection(
        &self,
        name: &str,
        min_size: u64,
        max_size: Option<u64>,
    ) -> Result<String, StopTestError> {
        let mut payload = cbor_map! {
            "name" => name,
            "min_size" => min_size
        };
        if let Some(max) = max_size {
            map_insert(&mut payload, "max_size", max);
        }
        let response = self.send_request("new_collection", &payload)?;
        match response {
            Value::Text(s) => Ok(s),
            _ => panic!(
                "Expected text response from new_collection, got {:?}",
                response
            ),
        }
    }

    fn collection_more(&self, collection: &str) -> Result<bool, StopTestError> {
        let response =
            self.send_request("collection_more", &cbor_map! { "collection" => collection })?;
        match response {
            Value::Bool(b) => Ok(b),
            _ => panic!("Expected bool from collection_more, got {:?}", response),
        }
    }

    fn collection_reject(&self, collection: &str, why: Option<&str>) -> Result<(), StopTestError> {
        let mut payload = cbor_map! {
            "collection" => collection
        };
        if let Some(reason) = why {
            map_insert(&mut payload, "why", reason.to_string());
        }
        self.send_request("collection_reject", &payload)?;
        Ok(())
    }

    fn new_pool(&self) -> Result<i128, StopTestError> {
        let response = self.send_request("new_pool", &cbor_map! {})?;
        match response {
            Value::Integer(i) => Ok(i.into()),
            other => panic!("Expected integer response for pool id, got {:?}", other),
        }
    }

    fn pool_add(&self, pool_id: i128) -> Result<i128, StopTestError> {
        let response = self.send_request("pool_add", &cbor_map! {"pool_id" => pool_id})?;
        match response {
            Value::Integer(i) => Ok(i.into()),
            other => panic!("Expected integer response for variable id, got {:?}", other),
        }
    }

    fn pool_generate(&self, pool_id: i128, consume: bool) -> Result<i128, StopTestError> {
        let response = self.send_request(
            "pool_generate",
            &cbor_map! {
                "pool_id" => pool_id,
                "consume" => consume,
            },
        )?;
        match response {
            Value::Integer(i) => Ok(i.into()),
            other => panic!("Expected integer response for variable id, got {:?}", other),
        }
    }

    fn mark_complete(&self, status: &str, origin: Option<&str>) {
        let origin_value = match origin {
            Some(s) => Value::Text(s.to_string()),
            None => Value::Null,
        };
        let mark_complete = cbor_map! {
            "command" => "mark_complete",
            "status" => status,
            "origin" => origin_value
        };
        let mut stream = self.stream.borrow_mut();
        let _ = stream.request_cbor(&mark_complete);
        let _ = stream.close();
    }

    fn test_aborted(&self) -> bool {
        self.aborted.get()
    }
}
