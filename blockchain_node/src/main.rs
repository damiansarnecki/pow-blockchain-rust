use clap::Parser;
use modules::blockchain::Blockchain;
use modules::cli::Config;
use modules::consensus::start_mining_loop;
use modules::network::start_networking;
use once_cell::sync::Lazy;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::{self};

mod modules;

pub static BLOCKCHAIN: Lazy<Arc<Mutex<Blockchain>>> = Lazy::new(|| {
    let port = Config::parse().port;
    Arc::new(Mutex::new(Blockchain::new(&format!(
        "blockchain_db_{}",
        port
    ))))
});


type SharedPeers = Arc<Mutex<Vec<TcpStream>>>;

fn main() {
    let config = Config::parse();
    let port = config.port;

    let peers: SharedPeers = Arc::new(Mutex::new(Vec::new()));
    let peers_clone: Arc<Mutex<Vec<TcpStream>>> = Arc::clone(&peers);


    thread::spawn(move || start_networking(port, peers_clone));

    {
        BLOCKCHAIN.lock().unwrap().initialize();
    };


    if config.mine {
        start_mining_loop(peers);
    }

    loop {}
}
