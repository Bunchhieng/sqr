use crate::db;
use crate::types::{
    ColumnInfo, DiagramData, DiagramTable, ForeignKeyInfo, IndexInfo, QueryResult, TableInfo,
};
use anyhow::Result;
use rusqlite::Connection;
use std::sync::mpsc;
use std::thread;

/// Messages sent to the worker thread
#[derive(Debug)]
pub enum WorkerMessage {
    LoadTables {
        include_internal: bool,
    },
    LoadTableRows {
        table_name: String,
        limit: usize,
        offset: usize,
    },
    ExecuteQuery {
        query: String,
        max_rows: Option<usize>,
    },
    GetTableInfo {
        table_name: String,
    },
    LoadSchema {
        table_name: String,
    },
    LoadDiagram,
    UpdateCell {
        table_name: String,
        row_index: usize,
        column_name: String,
        new_value: String,
    },
    Shutdown,
}

/// Responses sent back from the worker thread
#[derive(Debug)]
pub enum WorkerResponse {
    TablesLoaded {
        tables: Vec<TableInfo>,
    },
    TableRowsLoaded {
        result: QueryResult,
    },
    QueryExecuted {
        result: QueryResult,
    },
    TableInfoLoaded {
        info: TableInfo,
    },
    SchemaLoaded {
        columns: Vec<ColumnInfo>,
        indexes: Vec<IndexInfo>,
        foreign_keys: Vec<ForeignKeyInfo>,
    },
    DiagramLoaded {
        data: DiagramData,
    },
    Error {
        message: String,
    },
    CellUpdated,
}

/// Worker thread that handles database operations
pub struct Worker {
    sender: mpsc::Sender<WorkerMessage>,
    receiver: mpsc::Receiver<WorkerResponse>,
    handle: thread::JoinHandle<()>,
}

impl Worker {
    /// Create a new worker with a database connection
    pub fn new(conn: Connection) -> Self {
        let (tx, rx) = mpsc::channel();
        let (response_tx, response_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            let connection = conn;
            loop {
                match rx.recv() {
                    Ok(WorkerMessage::LoadTables { include_internal }) => {
                        match db::get_tables(&connection, include_internal) {
                            Ok(tables) => {
                                let _ = response_tx.send(WorkerResponse::TablesLoaded { tables });
                            }
                            Err(e) => {
                                let _ = response_tx.send(WorkerResponse::Error {
                                    message: format!("Failed to load tables: {}", e),
                                });
                            }
                        }
                    }
                    Ok(WorkerMessage::LoadTableRows {
                        table_name,
                        limit,
                        offset,
                    }) => {
                        match db::query::get_table_rows(&connection, &table_name, limit, offset) {
                            Ok(result) => {
                                let _ =
                                    response_tx.send(WorkerResponse::TableRowsLoaded { result });
                            }
                            Err(e) => {
                                let _ = response_tx.send(WorkerResponse::Error {
                                    message: format!("Failed to load rows: {}", e),
                                });
                            }
                        }
                    }
                    Ok(WorkerMessage::ExecuteQuery { query, max_rows }) => {
                        match db::query::execute_query(&connection, &query, max_rows) {
                            Ok(result) => {
                                let _ = response_tx.send(WorkerResponse::QueryExecuted { result });
                            }
                            Err(e) => {
                                // Error message is already formatted by db::query
                                let _ = response_tx.send(WorkerResponse::Error {
                                    message: format!("{}", e),
                                });
                            }
                        }
                    }
                    Ok(WorkerMessage::GetTableInfo { table_name }) => {
                        match db::get_table_info(&connection, &table_name) {
                            Ok(info) => {
                                let _ = response_tx.send(WorkerResponse::TableInfoLoaded { info });
                            }
                            Err(e) => {
                                let _ = response_tx.send(WorkerResponse::Error {
                                    message: format!("Failed to load table info: {}", e),
                                });
                            }
                        }
                    }
                    Ok(WorkerMessage::LoadSchema { table_name }) => {
                        match (
                            db::get_columns(&connection, &table_name),
                            db::get_indexes(&connection, &table_name),
                            db::get_foreign_keys(&connection, &table_name),
                        ) {
                            (Ok(columns), Ok(indexes), Ok(foreign_keys)) => {
                                let _ = response_tx.send(WorkerResponse::SchemaLoaded {
                                    columns,
                                    indexes,
                                    foreign_keys,
                                });
                            }
                            (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                                let _ = response_tx.send(WorkerResponse::Error {
                                    message: format!("Failed to load schema: {}", e),
                                });
                            }
                        }
                    }
                    Ok(WorkerMessage::LoadDiagram) => {
                        match db::get_tables(&connection, false) {
                            Ok(tables) => {
                                let mut diagram_tables = Vec::new();
                                for table in tables {
                                    match (
                                        db::get_columns(&connection, &table.name),
                                        db::get_foreign_keys(&connection, &table.name),
                                    ) {
                                        (Ok(columns), Ok(foreign_keys)) => {
                                            diagram_tables.push(DiagramTable {
                                                name: table.name,
                                                columns,
                                                foreign_keys,
                                            });
                                        }
                                        _ => {
                                            // Skip tables that fail to load
                                        }
                                    }
                                }
                                let _ = response_tx.send(WorkerResponse::DiagramLoaded {
                                    data: DiagramData {
                                        tables: diagram_tables,
                                    },
                                });
                            }
                            Err(e) => {
                                let _ = response_tx.send(WorkerResponse::Error {
                                    message: format!("Failed to load diagram: {}", e),
                                });
                            }
                        }
                    }
                    Ok(WorkerMessage::UpdateCell {
                        table_name,
                        row_index,
                        column_name,
                        new_value,
                    }) => {
                        match db::update_cell(
                            &connection,
                            &table_name,
                            row_index,
                            &column_name,
                            &new_value,
                        ) {
                            Ok(_) => {
                                let _ = response_tx.send(WorkerResponse::CellUpdated);
                            }
                            Err(e) => {
                                let _ = response_tx.send(WorkerResponse::Error {
                                    message: format!("Failed to update cell: {}", e),
                                });
                            }
                        }
                    }
                    Ok(WorkerMessage::Shutdown) => {
                        break;
                    }
                    Err(_) => {
                        // Channel closed, exit
                        break;
                    }
                }
            }
        });

        Self {
            sender: tx,
            receiver: response_rx,
            handle,
        }
    }

    /// Send a message to the worker
    pub fn send(&self, message: WorkerMessage) -> Result<()> {
        self.sender.send(message)?;
        Ok(())
    }

    /// Try to receive a response (non-blocking)
    pub fn try_recv(&self) -> Result<Option<WorkerResponse>> {
        match self.receiver.try_recv() {
            Ok(response) => Ok(Some(response)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => {
                Err(anyhow::anyhow!("Worker thread disconnected"))
            }
        }
    }

    /// Receive a response (blocking)
    #[allow(dead_code)]
    pub fn recv(&self) -> Result<WorkerResponse> {
        self.receiver
            .recv()
            .map_err(|e| anyhow::anyhow!("Worker thread disconnected: {}", e))
    }

    /// Shutdown the worker thread
    pub fn shutdown(self) -> Result<()> {
        self.sender.send(WorkerMessage::Shutdown)?;
        self.handle
            .join()
            .map_err(|_| anyhow::anyhow!("Worker thread panicked"))?;
        Ok(())
    }
}
