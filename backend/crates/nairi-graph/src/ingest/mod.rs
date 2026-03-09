use crate::db::Memgraph;
use nairi_ast::ir::{ApkIr, ClassIr};
use rsmgclient::ConnectParams;
use tokio::sync::{mpsc, oneshot};

use crate::mapping;

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("Actor channel closed")]
    ChannelClosed,
    #[error("Database error: {0}")]
    Database(String),
}

pub enum IngestMessage {
    InsertApk {
        apk: ApkIr,
        respond_to: oneshot::Sender<Result<(), IngestError>>,
    },
    InsertClasses {
        apk_id: String,
        classes: Vec<ClassIr>,
        respond_to: oneshot::Sender<Result<(), IngestError>>,
    },
    Stop,
}

#[derive(Clone)]
pub struct GraphActor {
    sender: mpsc::Sender<IngestMessage>,
}

impl GraphActor {
    pub fn new(sender: mpsc::Sender<IngestMessage>) -> Self {
        Self { sender }
    }

    pub async fn insert_apk(&self, apk: ApkIr) -> Result<(), IngestError> {
        let (send, recv) = oneshot::channel();
        let msg = IngestMessage::InsertApk {
            apk,
            respond_to: send,
        };
        self.sender
            .send(msg)
            .await
            .map_err(|_| IngestError::ChannelClosed)?;
        recv.await.map_err(|_| IngestError::ChannelClosed)?
    }

    pub async fn insert_classes(
        &self,
        apk_id: String,
        classes: Vec<ClassIr>,
    ) -> Result<(), IngestError> {
        let (send, recv) = oneshot::channel();
        let msg = IngestMessage::InsertClasses {
            apk_id,
            classes,
            respond_to: send,
        };
        self.sender
            .send(msg)
            .await
            .map_err(|_| IngestError::ChannelClosed)?;
        recv.await.map_err(|_| IngestError::ChannelClosed)?
    }
}

pub struct GraphActorSystem {
    receiver: mpsc::Receiver<IngestMessage>,
    host: String,
    port: u16,
}

impl GraphActorSystem {
    pub fn new(
        receiver: mpsc::Receiver<IngestMessage>,
        params: ConnectParams,
    ) -> Result<Self, String> {
        let host = params
            .host
            .clone()
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let port = params.port;
        Ok(Self {
            receiver,
            host,
            port,
        })
    }

    pub fn start(mut self) {
        tokio::task::spawn_blocking(move || {
            let params = ConnectParams {
                host: Some(self.host.clone()),
                port: self.port,
                lazy: false,
                autocommit: false,
                ..Default::default()
            };

            if let Err(e) = mapping::init_indices(&params) {
                eprintln!("Failed to initialize Memgraph indices: {}", e);
            }

            let mut db = match Memgraph::try_new(&params) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to connect to Memgraph: {}", e);
                    return;
                }
            };
            while let Some(msg) = self.receiver.blocking_recv() {
                match msg {
                    IngestMessage::InsertApk { apk, respond_to } => {
                        let result = mapping::insert_apk(&mut db, &apk)
                            .map_err(|e| IngestError::Database(e.to_string()));
                        let _ = db.commit();
                        let _ = respond_to.send(result);
                    }
                    IngestMessage::InsertClasses {
                        apk_id,
                        classes,
                        respond_to,
                    } => {
                        let result = mapping::insert_classes(&mut db, &apk_id, &classes)
                            .map_err(|e| IngestError::Database(e.to_string()));
                        let _ = db.commit();
                        let _ = respond_to.send(result);
                    }
                    IngestMessage::Stop => break,
                }
            }
        });
    }

    pub fn spawn_with(params: ConnectParams) -> Result<GraphActor, String> {
        let (sender, receiver) = mpsc::channel(1024);
        let system = GraphActorSystem::new(receiver, params)?;
        system.start();
        Ok(GraphActor::new(sender))
    }
}
