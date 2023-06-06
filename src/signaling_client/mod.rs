
use futures_channel::mpsc::{TrySendError,};
use futures_util::{pin_mut, StreamExt,};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use std::{
    thread,
};
use crate::aes::AES;

//pub type CallbackType = Arc<Mutex<dyn FnMut(String) + Send + 'static>>;
pub struct SignalingClient{
    username: String,
    address: String,
    port: u32,
    tx: Option<futures_channel::mpsc::UnboundedSender<Message>>,
    pub rx: Option<futures_channel::mpsc::UnboundedReceiver<Message>>,
    cipher: AES,
}
impl SignalingClient{

    /// Creates a new SignalingClient
    pub fn new(key:String, username:String, address: String, port: u32) -> Self{
        Self{
            username: username,
            address: address,
            port: port,
            tx: None,
            rx: None,
            cipher: AES::new(Some(key)),
        }
    }

    /// Creates a new thread that will listen for messages and send them to the caller thread through the `thread_tx` channel 
    /// The `thread_rx` parameter is the channel that the SignalingClient will use to send the recieved messages to the caller thread
    /// # Example
    /// ```no_run
    /// mod signaling_client;
    /// mod aes;
    /// use signaling_client::SignalingClient;
    /// use tokio_tungstenite::{tungstenite::protocol::Message};
    /// //This program will listen for messages and print them
    /// fn main(){
    ///    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
    ///    let mut client = SignalingClient::new(String::from("key"), String::from("username"), String::from("[::1]"), 8080);
    ///    client.run(tx);
    ///    loop{
    ///       let msg = rx.blocking_recv().unwrap();
    ///       println!("{}", msg);
    ///    }
    /// }
    /// ```
    pub fn run(&mut self, thread_tx: tokio::sync::mpsc::UnboundedSender<Message>){
        let (tx, rx) = futures_channel::mpsc::unbounded();
        self.tx = Some(tx);

        let addr = self.address.clone();
        let port = self.port.clone();
        let username = self.username.clone();
        let cipher = self.cipher.clone();

        thread::spawn(move ||{
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move{
                let connect_addr = format!("ws://{}:{}?username={}", addr, port, username);
                let url = url::Url::parse(&connect_addr).unwrap();
    
                let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
                println!("WebSocket handshake has been successfully completed");
                
                let (write, read) = ws_stream.split();
                
                //Map the caller thread channel to the websocket channel
                let rx_to_ws = rx.map(Ok).forward(write);
                
                let ws_to_stdout = {
                    read.for_each(|message| async {
                        //Decrypt message and send it through the caller thread channel
                        let data = message.unwrap().to_string();
                        let decrypted = cipher.decrypt(data);
                        let _ = thread_tx.send(Message::Text(decrypted));
                    })
                };
    
                pin_mut!(rx_to_ws,ws_to_stdout);
                futures_util::future::select(rx_to_ws,ws_to_stdout).await;
            });
        });
    }

    /// Encrypts and sends a message through the websocket
    /// # Example
    /// ```no_run
    /// mod signaling_client;
    /// mod aes;
    /// use signaling_client::SignalingClient;
    /// use tokio_tungstenite::{tungstenite::protocol::Message};
    /// //This program will listen for messages and print them
    /// fn main(){
    ///    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
    ///    let mut client = SignalingClient::new(String::from("key"), String::from("username"), String::from("[::1]"), 8080);
    ///    client.run(tx);
    ///    client.send_message("Hello World!".to_string());
    /// }
    /// ```
    pub fn send_message(&mut self, msg: String) -> Result<(), TrySendError<Message>>{
        let encrypted_message = self.cipher.encrypt(msg);
        self.tx.as_mut().unwrap().unbounded_send(Message::Text(encrypted_message))
    }

}