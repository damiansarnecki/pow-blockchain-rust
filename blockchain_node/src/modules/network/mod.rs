use super::block::Block;
use crate::{SharedPeers, BLOCKCHAIN};
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

pub fn handle_client(mut socket: TcpStream, peers: SharedPeers) {
    {
        let mut peer_lock = peers.lock().unwrap();
        peer_lock.push(socket.try_clone().unwrap());
    }

    let mut reader = BufReader::new(socket.try_clone().unwrap());
    let mut buffer = String::new();

    while reader.read_line(&mut buffer).unwrap_or(0) > 0 {
        let trimmed = buffer.trim();
        handle_message(trimmed, &mut socket);
        buffer.clear();
    }

    // Disconnect from client
    {
        let mut peer_lock = peers.lock().unwrap();
        if let Some(index) = peer_lock
            .iter()
            .position(|peer| peer.peer_addr().unwrap() == socket.peer_addr().unwrap())
        {
            peer_lock.remove(index);
            println!("Peer disconnected and removed.");
        }
    }
}

fn handle_message(message: &str, socket: &mut TcpStream) {
    let (flag, data) = match message.find(' ') {
        Some(pos) => (&message[..pos], &message[pos + 1..]),
        None => (message, ""),
    };

    match flag {
        "INV" => {
            println!("Received inventory!");

            let inv_package_deserialized: Vec<[u8; 32]> = serde_json::from_str(&data).unwrap();

            {
                let blockchain_lock = BLOCKCHAIN.lock().unwrap();

                for hash in inv_package_deserialized.iter() {
                    match blockchain_lock.get_block_index(hash) {
                        Some(_) => {
                        }
                        None => {
                            let hash_serialized = serde_json::to_string(hash).unwrap();
                            send_message(socket, &format!("GETDATA {}", &hash_serialized)).unwrap();
                        }
                    }
                }
            }
        }
        "CONNECT" => {
            send_message(socket, "ACK").unwrap();
        }
        "GETDATA" => {

            {
                let blockchain_lock = BLOCKCHAIN.lock().unwrap();

                let hash_deserialized = &serde_json::from_str(&data).unwrap();

                println!("{:?}", hash_deserialized);

                match blockchain_lock.get_block_index(hash_deserialized) {
                    Some(_) => {
                        let block_key = blockchain_lock
                            .get_block_index(hash_deserialized)
                            .unwrap()
                            .db_key;

                        match blockchain_lock.block_db.read_block(block_key) {
                            Some(block) => {
                                let block_serialized = serde_json::to_string(&block).unwrap();
                                send_message(socket, &format!("BLOCK {}", block_serialized))
                                    .unwrap();
                            }
                            None => {
                               
                            }
                        };
                    }
                    None => {
                        
                    }
                }
            }
        }
        "BLOCK" => {
            println!("Received block.");

            let block_deserialized: Block = serde_json::from_str(&data).unwrap();

            println!("{}", &data);

            {
                let mut blockchain_lock = BLOCKCHAIN.lock().unwrap();

                match blockchain_lock.get_block_index(&block_deserialized.hash) {
                    Some(_) => {
                    }
                    None => {
                        let prev_block_hash = block_deserialized.headers.previous_hash;

                        match blockchain_lock.get_block_index(&prev_block_hash) {
                            Some(_) => {
                                let saved_block_db_key =
                                    blockchain_lock.block_db.save_block(&block_deserialized);
                                blockchain_lock
                                    .add_block_to_index(&block_deserialized, saved_block_db_key);
                                blockchain_lock.connect_orphans(block_deserialized.hash);
                            }
                            None => {
                                let serialized_block_hash = serde_json::to_string(
                                    &block_deserialized.headers.previous_hash,
                                )
                                .unwrap();

                                blockchain_lock
                                    .orphan_blocks_map
                                    .insert(block_deserialized.hash, block_deserialized);
                                send_message(socket, &format!("GETDATA {}", serialized_block_hash))
                                    .expect("Failed to send message");
                            }
                        }
                    }
                }
            }
        }

        "ACK" => {
            println!("ACK {}", data)
        }
        _ => {
            println!("Unknown message type. Skipping.")
        }
    }
}

fn send_message(socket: &mut TcpStream, message: &str) -> Result<(), std::io::Error> {
    let message_with_newline = format!("{}\n", message);
    socket.write_all(message_with_newline.as_bytes())?;
    socket.flush()?;
    Ok(())
}

pub fn connect_to_peer(address: &str, peers: SharedPeers) {
    match TcpStream::connect(address) {
        Ok(stream) => {
            println!("Connected to: {}", address);

            let connect_message = "GETADDR\n";
            stream
                .try_clone()
                .unwrap()
                .write_all(connect_message.as_bytes())
                .unwrap();

            let peers_clone = Arc::clone(&peers);
            let listener_address = address.to_string();
            thread::spawn(move || handle_client(stream, peers_clone));
        }
        Err(e) => {
            println!("Error connecting to {}: {}", address, e);
        }
    }
}

pub fn scan_nodes(port: u16, range: u16, peers: SharedPeers) {
    for target_port in (port - range)..=(port + range) {
        if target_port != port {
            let address = format!("127.0.0.1:{}", target_port);
            connect_to_peer(&address, Arc::clone(&peers));
        }
    }
}

pub fn broadcast_inv(block: &Block, peers: SharedPeers) {
    {
        let peer_lock = peers.lock().unwrap();

        for peer in peer_lock.iter() {
            let socket = peer.try_clone(); 

            let mut inv_package = Vec::new();
            inv_package.push(block.hash);

            let inv_encoded = serde_json::to_string(&inv_package).unwrap();
            let inv_message = format!("INV {}\n", inv_encoded);

            if let Ok(mut stream) = socket {
                let _ = stream.write_all(inv_message.as_bytes());
            } else {
                println!("Failed to clone socket for peer {:?}", peer);
            }
        }
    }
}

pub fn start_networking(port: u16, peers: SharedPeers) {
    //Find first peers 
    scan_nodes(port, 2, Arc::clone(&peers));

    // Start listening
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Error");

    loop {
        match listener.accept() {
            Ok((socket, addr)) => {
                println!("New connection from: {}", addr);
                let peers_clone = Arc::clone(&peers);
                thread::spawn(move || handle_client(socket,peers_clone));
            }
            Err(e) => println!("Error accepting connection: {}", e),
        }
    }

   
}
