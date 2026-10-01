use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    time,
};

use crate::layers::{parser_layer::Command, storage_layer::Storage};

const HOST: &str = "127.0.0.1:6379";

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(&HOST).await?;
    let initial_storage = Storage::load_from_disk()?;
    let storage = Arc::new(Mutex::new(initial_storage));
    println!("[LOG]: Nori is running at {HOST}");

    let sweep_storage = Arc::clone(&storage);

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;

            let mut guard = sweep_storage.lock().unwrap();

            if let Err(err) = guard.sweep_expired_cache() {
                eprintln!("[ERROR]: Failed to sweep expired cache - {err}")
            };
        }
    });

    loop {
        let (socket, addr) = listener.accept().await?;
        let client_storage = Arc::clone(&storage);

        println!("[LOG]: Client {addr} connected.");

        tokio::spawn(async move {
            if let Err(err) = handle_client(socket, client_storage).await {
                eprintln!("[WARNING]: {err}");
            };
        });
    }
}

async fn handle_client(
    mut stream: TcpStream,
    storage: Arc<Mutex<Storage>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = stream.split();

    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();

        if reader.read_line(&mut line).await? == 0 {
            println!("[LOG]: Client disconnected.");
            break;
        };

        let req = line.trim();

        if req.is_empty() {
            continue;
        }

        println!("[MESSAGE]: {}", req);

        let response = match Command::parse(req) {
            Ok(command) => {
                println!("[COMMAND]: {command:?}");

                let mut storage = storage
                    .lock()
                    .map_err(|_| "Failed to acquire storage lock.")?;

                match command {
                    Command::Get { key } => match storage.get(&key) {
                        Some(data) => serde_json::json!({
                            "success": true,
                            "message": "Data found.",
                            "data": data
                        }),

                        None => serde_json::json!({
                            "success": false,
                            "message": "Data not found.",
                            "data": null
                        }),
                    },

                    Command::Set { key, value, ttl } => match storage.insert(key, value, ttl) {
                        Ok(_) => serde_json::json!({
                            "success": true,
                            "message": "Data stored successfully.",
                            "data": null
                        }),

                        Err(err) => serde_json::json!({
                            "success": false,
                            "message": format!(
                                "Failed to store data: {err}"
                            ),
                            "data": null
                        }),
                    },

                    Command::SetX { key, value, ttl } => {
                        match storage.insert(key, value, Some(ttl)) {
                            Ok(_) => serde_json::json!({
                                "success": true,
                                "message": "Data stored successfully with TTL.",
                                "data": {
                                    "ttl": ttl
                                }
                            }),

                            Err(err) => serde_json::json!({
                                "success": false,
                                "message": format!(
                                    "Failed to store data: {err}"
                                ),
                                "data": null
                            }),
                        }
                    }

                    Command::Delete { key } => match storage.delete(&key) {
                        Ok(true) => serde_json::json!({
                            "success": true,
                            "message": "Data deleted successfully.",
                            "data": null
                        }),

                        Ok(false) => serde_json::json!({
                            "success": false,
                            "message": "Data not found.",
                            "data": null
                        }),

                        Err(err) => serde_json::json!({
                            "success": false,
                            "message": format!(
                                "Failed to delete data: {err}"
                            ),
                            "data": null
                        }),
                    },

                    Command::Stats => serde_json::json!({
                        "success": true,
                        "message": "Statistics retrieved successfully.",
                        "data": {
                            "cache_hits": storage.cache_hits,
                            "cache_misses": storage.cache_misses,
                            "total_keys": storage.store.len()
                        }
                    }),

                    Command::Help => serde_json::json!({
                        "success": true,
                        "message": "Available commands.",
                        "data": {
                            "GET": "GET <KEY>",
                            "SET": "SET <KEY> <VALUE> [EX <SECONDS>]",
                            "SETX": "SETX <KEY> <VALUE> <SECONDS>",
                            "DELETE": "DELETE <KEY>",
                            "DEL": "DEL <KEY>",
                            "STATS": "STATS",
                            "HELP": "HELP"
                        }
                    }),

                    Command::Unknown => serde_json::json!({
                        "success": false,
                        "message": "Unknown command.",
                        "data": null
                    }),
                }
            }

            Err(err) => serde_json::json!({
                "success": false,
                "message": err.to_string(),
                "data": null
            }),
        };

        let response = serde_json::to_string_pretty(&response)?;

        writer.write_all(response.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }

    Ok(())
}
