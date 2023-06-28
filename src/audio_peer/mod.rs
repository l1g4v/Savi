// SPDX-FileCopyrightText: Copyright 2023 Savi
// SPDX-License-Identifier: GPL-3.0-only 

use log::debug;
use tokio::runtime::Runtime;
use tokio::net::UdpSocket;
use std::{sync::{Arc, Mutex, atomic::{AtomicBool, Ordering, AtomicU64}}, thread, ops::Add};
use miniaudio::DeviceConfig;
use crate::audio::playback::AudioPlayback;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// AudioPeer allows for sending and receiving audio packets between two peers
/// ## Example with ```audio::playback``` and ```audio::capture```
/// ```no_run
/// //This program takes 5 arguments: uid bind connect mic_id speaker_id
/// //Takes the capture queue elements and send it through the socket
/// //while AudioPeer handles packets and playback
/// 
/// use std::env;
/// 
/// mod audio;
/// mod audio_peer;
/// 
/// use audio::playback::AudioPlayback;
/// use audio::capture::AudioCapture;
/// use audio_peer::AudioPeer;
/// 
/// fn main(){
/// // args: uid bind connect mic_id speaker_id
/// let arg: Vec<String> = env::args().collect();
/// let args = arg[1..].to_vec();
/// //print devices
/// audio::Audio::print_devices();
/// println!("{:?}", args);
/// if args.len() != 5 {
///    panic!("invalid number of arguments");
/// }
/// 
/// let inputs = audio::Audio::get_input_devices();
/// let outputs = audio::Audio::get_output_devices();
/// let audio_capture = AudioCapture::new(inputs[args[3].parse::<usize>().unwrap()].clone().1,
/// 1, 48_000, 128_000, 100.0);
/// audio_capture.start();
/// 
/// let playback_config = AudioPlayback::create_config(outputs[args[4].parse::<usize>().unwrap()].clone().1,2,48_000);
/// let mut peer = AudioPeer::new(args[0].parse::<u8>().unwrap(),args[1].clone());
/// peer.connect(args[2].clone(), playback_config);
/// let capture_arc = audio_capture.get_capture_arc();
/// 
/// loop{
///     let (mutex, cvar) = &*capture_arc;
///     let mut queue = mutex.lock().unwrap();
///     while queue.is_empty() {
///         queue = cvar.wait(queue).unwrap();
///     }
///     let (packet_number, payload) = queue.pop_front().unwrap();
///         peer.send(payload);
///     }
/// }
/// ```
pub struct AudioPeer {
    ready: Arc<AtomicBool>,
    packet_count: Arc<AtomicU64>,
    volume: Arc<Mutex<u8>>,
    udpsocket: Arc<Mutex<std::net::UdpSocket>>,
}
impl AudioPeer {
    /// Creates a new AudioPeer
    /// # Arguments
    /// * `bind` - The address to bind to
    pub fn new(bind: String) -> AudioPeer {
        AudioPeer {
            packet_count: Arc::new(AtomicU64::new(0)),
            ready: Arc::new(AtomicBool::new(false)),
            volume: Arc::new(Mutex::new(100)),
            //tk_socketqueue: Arc::new(Mutex::new(BinaryHeap::new())),
            udpsocket: Arc::new(Mutex::new(
                std::net::UdpSocket::bind(bind).expect("couldn't bind to address"),
            )),
        }
    }

    // TODO: change playback_config to an AudioPlayback object
    /// Connects to a peer
    /// # Arguments
    /// * `addr` - The address to connect to
    /// * `playback_config` - The configuration for the playback device
    pub fn connect(&self, addr: String, playback_config: DeviceConfig) {
        //self.target_username = username;
        //measure time
        //let start = std::time::Instant::now();
        
        let _ = self.udpsocket.lock().unwrap().connect(&addr).expect("couldn't connect to address");
        //connect to server
        let socket_clone = self.udpsocket.lock().unwrap().try_clone().unwrap();

        let volume = self.volume.clone();
        let mut audio_playback = AudioPlayback::new(playback_config);
        let ready = self.ready.clone();
        //Avoids a weird bug where the cpu usage grows when one of the two peers never receives a packet
        self.udpsocket.lock().unwrap().send(&[1]).unwrap();
        
        
        thread::spawn(move || {
            audio_playback.start();
            let rt = Runtime::new().unwrap();
            let mut buffer: BinaryHeap<Reverse<(u64, Vec<u8>)>> = BinaryHeap::new();
            //TODO[LATENCY]: this may or may not be implemented
            //let mut start = std::time::Instant::now();
            
            let playback_arc = audio_playback.get_playback_arc();
            
            rt.block_on(async move {
                let tk_socket = UdpSocket::from_std(socket_clone).unwrap();
                let mut data = [0; 1024];
                loop {
                    match tk_socket.try_recv(&mut data[..]) {
                        Ok(n) => {
                            if n == 1 && !ready.load(Ordering::Relaxed){
                                /* TODO[LATENCY]: this may or may not be implemented
                                let _ = tk_socket.send(&[1]).await;
                                let elapsed = start.elapsed();
                                println!("Latency: {}ms", elapsed.as_millis());
                                start = std::time::Instant::now();
                                */
                                debug!("Ready");
                                ready.store(true, Ordering::Relaxed);
                                tk_socket.send(&[1]).await.unwrap();
                                continue;
                            }
                            if n < 8 {
                                continue;
                            }
                            //Deserialize packet count
                            let received = data[..n].to_vec();
                            let recv_packet_count: u64 = bincode::deserialize(&received[n - 8..]).unwrap();

                            //Push to playback queue
                            let mut opus = data[..n - 8].to_vec();
                            opus.push(*volume.lock().unwrap());
                            let voice = (recv_packet_count, opus);
                            buffer.push(Reverse(voice));
                        }
                        // False-positive, continue
                        Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock || e.kind() == tokio::io::ErrorKind::ConnectionRefused => {
                            println!("false positive");
                        }
                        Err(e) => {
                            panic!("{:?}",e.kind());
                        }
                    }
                    //the "is this a jitter buffer¿" implementation
                    if buffer.len() > 1 {
                        let (mutex, cvar) = &*playback_arc;
                        let mut play_queue = mutex.lock().unwrap();
                        while !buffer.is_empty() {
                            let payload = buffer.pop().unwrap().0.1;
                            play_queue.push(payload);
                            cvar.notify_all();
                        }
                    }
                }
            });
        });
    }

    /// Sends a voice packet through the socket.
    /// The packet is serialized as follows:
    /// <opus packet variable size><packet number 8 bytes>
    /// # Arguments
    /// * `data` - An opus packet
    /// # Returns
    /// * `usize` - The number of bytes sent
    /// # Errors
    /// * `std::io::Error` - If the peer is not ready
    pub fn send(&self, data: Vec<u8>) -> std::io::Result<usize> {
        if !self.is_ready(){
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Peer not ready"));
        }
        let packet_count = self.packet_count.fetch_add(1, Ordering::Relaxed);
        let mut serialized = bincode::serialize(&(packet_count-1)).unwrap();
        
        let mut payload = data;
        payload.append(serialized.as_mut());
        self.udpsocket
            .lock()
            .unwrap()
            .send(&payload)
    }

    pub fn change_volume(&self, volume: u8) {
        *self.volume.lock().unwrap() = volume;
    }
    
    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }
}
