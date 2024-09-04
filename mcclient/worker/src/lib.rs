use base64::{engine::general_purpose, Engine as _};
use std::path::Path;

use kinode_process_lib::{
    await_message, call_init, get_blob, println, spawn,
    vfs::{create_file, open_dir, open_file, DirEntry, FileType, SeekFrom, VfsAction, VfsRequest},
    Address, Message, Request, OnExit, our_capabilities
};

use files_lib::encryption::{encrypt_data, CHUNK_SIZE};
use files_lib::structs::{WorkerRequest, WorkerStatus};
use files_lib::read_nested_dir_light;

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});

fn handle_message(our: &Address, receive_chunks_to_dir: &mut String) -> anyhow::Result<bool> {
    let message = await_message()?;

    if let Message::Request { ref body, .. } = message {
        let request = serde_json::from_slice::<WorkerRequest>(body)?;
        match request {
            // we will be sending chunks to `target_worker`, encrypting w/ `password_hash`, from directory `sending_from_dir`
            // if password_hash is None, we will not be encrypting
            WorkerRequest::InitializeSenderWorker {
                target_worker,
                sending_dir,
                password,
            } => {
                println!("sending_dir: {}", sending_dir);
                // send to ourself for testing purposes
                let target_worker: Address = target_worker.unwrap_or(our.clone());

                println!("worker: got initialize request");
                let dir_entry = DirEntry {
                    path: sending_dir.clone(),
                    file_type: FileType::Directory,
                };

                // outputs map(path -> contents) where contents are empty, 
                // a flattened version of the nested dir
                let dir = read_nested_dir_light(dir_entry)?;

                // send each file from the folder to the server
                for path in dir.keys() {
                    let mut active_file = open_file(path, false, Some(5))?;

                    // we have a target, chunk the data, and send it.
                    let size = active_file.metadata()?.len;
                    let _pos = active_file.seek(SeekFrom::Start(0))?;

                    let sending_dir_path = Path::new(sending_dir.as_str());
                    let parent = sending_dir_path.parent().unwrap_or(sending_dir_path);
                    let parent_str = parent.to_str().unwrap_or(sending_dir.as_str());

                    let file_path =
                        // encrypts file name
                        if let Some(password) = password.clone() {
                            // path: e.g. folder_transfer:astronaut.os/from/Obsidian Vault/file.md
                            // we are sending: GAXPVM...0pihtLlOiu_E3A==

                            let p = if path.starts_with(parent_str) {
                                let rest_of_path = &path[parent_str.len()..].to_string();
                                let encrypted_vec = &encrypt_data(
                                    rest_of_path.as_bytes(),
                                    password.as_str(),
                                );
                                format!("/{}", general_purpose::URL_SAFE.encode(&encrypted_vec))
                            } else {
                                return Err(anyhow::anyhow!(
                                    "Path does not start with the expected prefix"
                                ));
                            };
                            p
                        } 
                        // doesnt encrypt file name
                        else {
                            // if full path is folder_transfer:astronaut.os/from/Obsidian Vault/file.md
                            // we are sending Obsidian Vault/file.mdđ
        
                            let p = if path.starts_with(parent_str) {
                                path[parent_str.len()..].to_string()
                            } else {
                                return Err(anyhow::anyhow!(
                                    "Path does not start with the expected prefix"
                                ));
                            };
                            p
                        };


                    // chunking and sending
                    //
                    // handling the edge case if there is 0 bytes, 
                    // we still want to send one chunk to make sure the empty file is transferred
                    let num_chunks = if size != 0 {
                        (size as f64 / CHUNK_SIZE as f64).ceil() as u64
                    } else {
                        1
                    };

                    for i in 0..num_chunks {
                        let offset = i * CHUNK_SIZE;
                        let length = CHUNK_SIZE.min(size - offset); // size=file size
                        let mut buffer = vec![0; length as usize];
                        let pos = active_file.seek(SeekFrom::Current(0))?;
                        active_file.read_at(&mut buffer)?;
                        
                        if let Some(pw_hash) = password.clone() {
                            buffer = encrypt_data(&buffer, pw_hash.as_str());
                        }

                        Request::new()
                            .body(serde_json::to_vec(&WorkerRequest::Chunk {
                                file_path: file_path.clone(),
                                done: false,
                                encrypted: password.is_some(),
                            })?)
                            .target(target_worker.clone())
                            .blob_bytes(buffer.clone())
                            .send()?;
                    }
                }
                println!("worker: sent everything");
                Request::new()
                    .body(serde_json::to_vec(&WorkerRequest::Chunk {
                        file_path: "".to_string(),
                        done: true,
                        encrypted: password.is_some(),
                    })?)
                    .target(target_worker.clone())
                    .send()?;

                return Ok(true);
            }

            // we will be receivng chunks to directory `receive_to_dir`
            WorkerRequest::InitializeReceiverWorker { receive_to_dir } => {

                // start receiving data
                let full_path = receive_to_dir;
                *receive_chunks_to_dir = full_path.clone();
                
                println!("starting to receive data for dir: {}", full_path);

                // removing the dir, and creating a fresh one
                let request: VfsRequest = VfsRequest {
                    path: full_path.to_string(),
                    action: VfsAction::RemoveDirAll,
                };
                let _message = Request::new()
                    .target(("our", "vfs", "distro", "sys"))
                    .body(serde_json::to_vec(&request)?)
                    .send_and_await_response(5)?;

                let request: VfsRequest = VfsRequest {
                    path: full_path.to_string(),
                    action: VfsAction::CreateDirAll,
                };
                let _message = Request::new()
                    .target(("our", "vfs", "distro", "sys"))
                    .body(serde_json::to_vec(&request)?)
                    .send_and_await_response(5)?;
            }

            // every time we receive a chunk, append to the file
            WorkerRequest::Chunk { file_path, done, encrypted } => {
                if done == true {
                    return Ok(true);
                }
                
                println!("got file_path: {}", file_path);
                let blob = get_blob();
                
                let path_to_dir = receive_chunks_to_dir; // just skipping the initial '/'
                let file_path = format!("{}{}", path_to_dir, &file_path);

                let file_path_as_path = Path::new(file_path.as_str());
                let parent = file_path_as_path.parent().unwrap_or(file_path_as_path);
                let parent_str = parent.to_str().unwrap_or(file_path.as_str());
                let request: VfsRequest = VfsRequest {
                    path: parent_str.to_string(),
                    action: VfsAction::CreateDirAll,
                };
                let _message = Request::new()
                    .target(("our", "vfs", "distro", "sys"))
                    .body(serde_json::to_vec(&request)?)
                    .send_and_await_response(5)?;

                
                let bytes = match blob {
                    Some(blob) => blob.bytes,
                    None => {
                        return Err(anyhow::anyhow!("worker: receive error: no blob"));
                    }
                };
                
                // manually creating file if doesnt exist, since open_file(create:true) has an issue
                let dir = open_dir(parent_str, false, Some(5))?;

                let entries = dir.read()?;
                if entries.contains(&DirEntry {
                    path: file_path.clone(),
                    file_type: FileType::File,
                }) {
                } else {
                    let _file = create_file(&file_path, Some(5))?;
                }

                let mut file = open_file(&file_path, false, Some(5))?;
                file.append(&bytes)?;
            }
        }
    }
    Ok(false)
}

call_init!(init);
fn init(our: Address) {
    println!("worker: begin");
    let start = std::time::Instant::now();

    // directory to which we will be storing received data
    let mut receive_chunks_to_dir = String::new();

    loop {
        match handle_message(&our,&mut receive_chunks_to_dir) {
            Ok(exit) => {
                if exit {
                    println!(
                        "worker: done: , took {:?}",
                        start.elapsed()
                    );
                    let _ = Request::new()
                    .body(serde_json::to_vec(&WorkerStatus::Done).unwrap())
                    .target(
                        Address::new(
                        our.node(),
                        ("mcclient", "mcclient", "basilesex")))
                    .send();
                    break;
                }
            }
            Err(e) => {
                println!("worker error: {:?}", e);
            }
        };
    }
}
