// SPDX-FileCopyrightText: Copyright 2023 Savi
// SPDX-License-Identifier: GPL-3.0-only 

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{
        handshake::server::{Request, Response},
        protocol::{Message}
    },
};
use crate::aes::AES;

//pub type CallbackType = Arc<Mutex<dyn Fn(String) + Send + 'static>>;

type PeerMap = Arc<Mutex<HashMap<SocketAddr, Peer>>>;
struct Peer{
    pub id: usize,
    pub username: String,
    pub tx: UnboundedSender<Message>,
}

pub struct SignalingServer{
    address: String,
    port: u32,
    cipher_key: Option<String>,
}
impl SignalingServer{
    /// Creates a new SignalingServer
    pub fn new(listen_addr: String, listen_port: u32) -> Self{
        SignalingServer{
            address: listen_addr,
            port: listen_port,
            cipher_key: None
        }
    }
    /// Creates a new thread where a websocket server will be running and listening for new connections
    /// The server will send a message to all the connected clients
    /// When a new client connects, the server will send an id to the client letting it know what id it has
    /// 
    /// # Example
    /// ```no_run
    /// #[macro_use]
    /// extern crate log;
    /// mod signaling_server;
    /// use signaling_server::SignalingServer;
    /// mod aes;
    /// 
    /// fn main(){
    ///    env_logger::init();
    ///    let mut server = SignalingServer::new("[::]".to_string(), 8080);
    ///    server.run();
    ///    loop{};
    /// }
    /// ```
    pub fn run(&mut self){
        let rt = tokio::runtime::Runtime::new().unwrap();
        let addr = self.address.clone();
        let port = self.port.clone();
        let cipher = AES::new(None);
        self.cipher_key = Some(cipher.get_key().clone());

        debug!("Spawning thread");
        thread::spawn(move ||{
            let addr = format!("{}:{}", addr, port);
            let state = PeerMap::new(Mutex::new(HashMap::new()));
            // Create the event loop and TCP listener we'll accept connections on.
            rt.block_on(async move{
                let try_socket = TcpListener::bind(&addr).await;
                let listener = try_socket.expect("Failed to bind");
                debug!("Listening on: {}", addr);
            
                // Let's spawn the handling of each connection in a separate task.
                while let Ok((stream, addr)) = listener.accept().await {
                    tokio::spawn(Self::handle_connection(cipher.clone(),state.clone(), stream, addr));
                }
            });            
        });
    }
    
    /// Returns the cipher key generated by the server. **If the server has not been started yet, it will return an empty string**
    pub fn get_cipher_key(&self) -> String{
        self.cipher_key.clone().unwrap_or_default()
    }

    /// Connection handler.
    /// When a new client connects, it will send it an id and add it to the peer map
    /// When a client sends a message, it will broadcast it to all other clients
    async fn handle_connection(cipher: AES, peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
        debug!("Incoming TCP connection from: {}", addr);
        let (tx, rx) = unbounded();
        let callback = |req: &Request, response: Response| {
            let username = get_username(req.uri().query().unwrap().to_string());
            debug!("Added new user: {}({}) to the signaling server", username,addr);    
            let new_id = peer_map.lock().unwrap().len();
            let encrypted_welcome = cipher.encrypt(String::from("id¬")+ &(new_id).to_string()+&"\n".to_string());
            let _ = tx.unbounded_send(Message::Text(encrypted_welcome));
            peer_map.lock().unwrap().insert(addr, Peer { username: username, tx: tx, id: new_id  });
            
            Ok(response)
        };
    
        let ws_stream = accept_hdr_async(raw_stream,callback)
            .await
            .expect("Error during the websocket handshake occurred");
        
    
        let (outgoing, incoming) = ws_stream.split();
        let broadcast_incoming = incoming.try_for_each(|msg| {
            let peers = peer_map.lock().unwrap();
            
            let incoming_username = peers.get(&&addr).unwrap().username.clone();    
            debug!("Received a message from {}({}): {}", incoming_username,addr, msg.to_text().unwrap());

            //Broadcast the message to all other peers except the sender
            let broadcast_recipients =
                peers.iter().filter(|(peer_addr, _)| peer_addr != &&addr).map(|(_, ws_sink)| ws_sink);
    
            for recp in broadcast_recipients {
                recp.tx.unbounded_send(msg.clone()).unwrap();
            }
    
            future::ok(())
        });
    
        let receive_from_others = rx.map(Ok).forward(outgoing);
    
        pin_mut!(broadcast_incoming, receive_from_others);
        future::select(broadcast_incoming, receive_from_others).await;
    
        debug!("{} disconnected", &addr);
        peer_map.lock().unwrap().remove(&addr);
    }
    
    
}

//utils

/// You know what this does
fn get_username(query: String) -> String{
    let mut username = String::new();
    let qe = query.split("&");
    for q in qe{
        let mut q = q.split("=");
        let key = q.next().unwrap();
        let value = q.next().unwrap();
        if key == "username"{
            username = urlencoding::decode(&value.to_string()).expect("UTF-8").to_string();
        }
    }
    username
}