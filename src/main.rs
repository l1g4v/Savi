// SPDX-FileCopyrightText: Copyright 2023 Savi
// SPDX-License-Identifier: GPL-3.0-only 

#[macro_use]
extern crate log;

use miniaudio::{Context, Backend};
use slint::{Model, SharedString};
use std::fmt::format;
use std::net::UdpSocket;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicI32};
use std::sync::{Arc, Mutex, mpsc};
use std::sync::mpsc::channel;

use std::thread;
use tokio::runtime::Runtime;
slint::include_modules!();

mod aes;
mod signaling;
mod audio;
use audio::playback::AudioPlayback;
use audio::capture::AudioCapture;
use audio::Audio;
mod audio_peer;
use audio_peer::AudioPeer;
use signaling::server::SignalingServer;
use signaling::client::SignalingClient;

use crate::audio::capture;

struct PeerListData {
    peers: Rc<slint::VecModel<Peer>>,
}

impl PeerListData {
    fn add_peer(&self, id: i32, name: slint::SharedString, adress: slint::SharedString) {
        self.peers.push(Peer {
            id: id,
            name: name,
            ip: adress,
        })
    }
}


fn main() {
    //
    #[cfg(target_os="windows")]
    let backend_list: Vec<SharedString> = vec![
        "Wasapi".into(),
        "DirectSound".into(),
        "WinMM".into(),
    ];
    #[cfg(target_os="linux")]
    let backend_list: Vec<SharedString> = vec![
        "PulseAudio".into(),
        "ALSA".into(),
        "JACK".into(),
    ];
    #[cfg(any(target_os="openbsd", target_os="freebsd", target_os="netbsd"))]
    let backend_list: Vec<SharedString> = vec![
        "sndio".into(),
        "Audio4".into(),
        "OSS".into(),
    ];
    #[cfg(target_os="macos")]
    let backend_list: Vec<SharedString> = vec![
        "CoreAudio".into(),
        "PulseAudio".into(),
        "JACK".into(),
    ];
    //

    env_logger::init();
    let app = App::new().unwrap();
    let app_clone = app.clone_strong();
    let app_clone2 = app.clone_strong();
    let app_clone3 = app.clone_strong();
    let app_clone4 = app.clone_strong();
    let app_clone5 = app.clone_strong();
    let app_clone6 = app.clone_strong();
    let app_clone7 = app.clone_strong();
    let app_weak = app.as_weak();

    let playback_id = Arc::new(Mutex::new(String::new()));
    let playback_id_clone = playback_id.clone();
    let playback_id_clone2 = playback_id.clone();
    let playback_id_clone3 = playback_id.clone();
    let playback_id_clone4 = playback_id.clone();

    let default_backend = Audio::backend_from_text(backend_list[0].to_string());

    let backend_arc = Arc::new(Mutex::new(backend_list[0].clone().to_string()));
    let backend_arc2 = backend_arc.clone();
    let backend_arc3 = backend_arc.clone();
    let backend_arc4 = backend_arc.clone();

    let capture_devices = Audio::get_input_devices(Some(default_backend));
    let playback_devices = Audio::get_output_devices(Some(default_backend));

    let mut capture_devices_str = Vec::new();
    for device in capture_devices.iter() {
        capture_devices_str.push(slint::SharedString::from(device.0.clone()));
    }
    let mut playback_devices_str = Vec::new();
    for device in playback_devices.iter() {
        playback_devices_str.push(slint::SharedString::from(device.0.clone()));
    }

    let crc = Rc::new(slint::VecModel::from(capture_devices_str.clone()));
    let prc = Rc::new(slint::VecModel::from(playback_devices_str.clone()));
    let cbrc = Rc::new(slint::VecModel::from(backend_list.clone()));

    app.global::<AudioDevices>().set_capture_devices(crc.into());
    app.global::<AudioDevices>().set_playback_devices(prc.into());
    app.global::<AudioDevices>().set_capture_backend(backend_list[0].clone());
    app.global::<AudioDevices>().set_playback_backend(backend_list[0].clone());
    app.global::<AudioDevices>().set_backends(cbrc.into());

    *playback_id_clone.lock().unwrap() = playback_devices[0].0.clone();

    let (capture_tx, capture_rx) = mpsc::channel::<Vec<u8>>();
    let capture_rx_arc = Arc::new(Mutex::new(capture_rx));
    let capture_rx_arc2 = capture_rx_arc.clone();

    let capture_device: Arc<Mutex<AudioCapture>> = Arc::new(Mutex::new(AudioCapture::new(default_backend, capture_devices[0].1.clone(), 
    1, 48_000, 96_000, 0, capture_tx.clone())));
    capture_device.lock().unwrap().start();

    let bind = capture_device.lock().unwrap().get_conn_addr();
    let connect = capture_device.lock().unwrap().get_queue_addr();
    info!("Bind: {}", bind);
    info!("Connect: {}", connect);

    let capture_device_clone = capture_device.clone();
    let capture_device_clone2 = capture_device.clone();
    let capture_device_clone3 = capture_device.clone();
    let capture_device_clone4 = capture_device.clone();
    let capture_device_clone5 = capture_device.clone();
    let capture_device_clone6 = capture_device.clone();

    app.global::<AudioDevices>().on_set_capture(move |id|{
        let threshold = app_clone.global::<AudioDevices>().get_input_threshold();
        println!("Capture device set to {}", capture_devices_str[id as usize].as_str());
        capture_device_clone.lock().unwrap().stop();
        let backend_str = app_clone6.global::<AudioDevices>().get_capture_backend().to_string();
        let backend = Audio::backend_from_text(backend_str);
        *capture_device_clone.lock().unwrap() = AudioCapture::new(backend, capture_devices[id as usize].1.clone(), 
        1, 48_000, 96_000, threshold, capture_tx.clone());
        capture_device_clone.lock().unwrap().start();
    });

    app.global::<AudioDevices>().on_in_settings(move || {
        let cc = capture_device_clone2.clone();
        let c2 = app_weak.clone();
        thread::spawn(move ||{
            let run:Arc<AtomicBool> = Arc::new(true.into());
            while run.load(std::sync::atomic::Ordering::Relaxed){
                let th = cc.lock().unwrap().get_intensity();
                //c2.lock().unwrap().global::<AudioDevices>().set_input_intensity(th);
                c2.upgrade_in_event_loop(move |handle| handle.global::<AudioDevices>().set_input_intensity(th)).unwrap();
                let x = run.clone();
                c2.upgrade_in_event_loop(move |handle| {x.store(handle.global::<AudioDevices>().get_on_settings(), std::sync::atomic::Ordering::Relaxed)}).unwrap();
                //sleep 10ms
                thread::sleep(std::time::Duration::from_millis(10));
            }
        });
    });

    app.global::<AudioDevices>().on_set_bitrate(move |bitrate_str|{
        let bitrate = bitrate_str.parse::<i32>().unwrap();
        capture_device_clone3.lock().unwrap().set_encoder_bitrate(bitrate);
    });

    app.global::<AudioDevices>().on_set_playback(move |name|{
        *playback_id_clone.lock().unwrap() = name.to_string();
        //playback_id_clone.store(idx, std::sync::atomic::Ordering::Relaxed);
    });

    app.global::<AudioDevices>().on_set_capture_backend(move |backend|{
        *backend_arc2.lock().unwrap() = backend.to_string();
        let backend_obj = Audio::backend_from_text(backend.to_string());
        let capture_devices = Audio::get_input_devices(Some(backend_obj));

        let mut capture_devices_str = Vec::new();
        for device in capture_devices.iter() {
            capture_devices_str.push(slint::SharedString::from(device.0.clone()));
        }

        let vrc = Rc::new(slint::VecModel::from(capture_devices_str.clone()));
        app_clone4.global::<AudioDevices>().set_capture_devices(vrc.into());
        //backend_arc.store(backend.to_string(), std::sync::atomic::Ordering::Relaxed);
    });

    app.global::<AudioDevices>().on_set_playback_backend(move |backend|{
        *backend_arc3.lock().unwrap() = backend.to_string();
        let backend_obj = Audio::backend_from_text(backend.to_string());
        let playback_devices = Audio::get_output_devices(Some(backend_obj));

        let mut playback_devices_str = Vec::new();
        for device in playback_devices.iter() {
            playback_devices_str.push(slint::SharedString::from(device.0.clone()));
        }

        let vrc = Rc::new(slint::VecModel::from(playback_devices_str.clone()));
        app_clone5.global::<AudioDevices>().set_playback_devices(vrc.into());
        //backend_arc.store(backend.to_string(), std::sync::atomic::Ordering::Relaxed);
    });

    //Network
    let cs_instance: Arc<Mutex<(Option<SignalingClient>,Option<SignalingServer>)>> = Arc::new(Mutex::new((None, None)));
    let cs_instance_clone = cs_instance.clone();
    let cs_instance_clone2 = cs_instance.clone();

    app.global::<Signaling>().on_create(move ||{
        let backend = backend_arc.lock().unwrap().clone();
        let username = app_clone2.global::<SelfPeer>().get_name().to_string();
        
        let server = SignalingServer::new(username);
        let listen = server.get_listen_address();
        let key = server.get_cipher_key();
        
        app_clone2.global::<Signaling>().set_address(slint::SharedString::from(listen));
        app_clone2.global::<Signaling>().set_key(slint::SharedString::from(key));
        app_clone2.global::<Signaling>().set_hosting(true);

        let playback_name = playback_id_clone3.lock().unwrap().clone();

        let rx = capture_rx_arc.clone();
        thread::spawn(move ||{
            let server_arc = Arc::new(server);
            let server_arc2 = server_arc.clone();
            let rx2 = rx.clone();
            thread::spawn(move ||{
                server_arc2.run(backend, playback_name);
            });
            thread::spawn(move||{
                let c_rx = rx2.lock().unwrap();
                loop{
                    //TODO: Implement some queue/buffer/idk
                    let packet = c_rx.recv();
                    if packet.is_err(){
                        continue;
                    }
                    let p = packet.unwrap();
                    server_arc.send_opus(p);
                }
            });
        });

    });
    //TODO: implement a socket to read from AudioCapture
    app.global::<Signaling>().on_connect(move |addr, key|{
        let backend = backend_arc4.lock().unwrap().clone();
        let username = app_clone3.global::<SelfPeer>().get_name().to_string();

        let client = SignalingClient::new(username, addr.to_string(), key.to_string());
        //cs_instance_clone2.lock().unwrap().0 = Some(client);
        let playback_name = playback_id_clone4.lock().unwrap().clone();
        //let cs_cinstance = cs_instance_clone2.clone();        

        println!("Connecting to {}", addr.as_str());
        println!("Key: {}", key.as_str());
        app_clone3.global::<Signaling>().set_connected(true);

        //let cd = capture_device_clone5.clone();
        let bind = capture_device_clone5.lock().unwrap().get_conn_addr();
        let connect = capture_device_clone5.lock().unwrap().get_queue_addr();
        info!("Bind: {}", bind);
        info!("Connect: {}", connect);
        //let cs_cinstance2 = cs_instance_clone2.clone();
        let rx = capture_rx_arc2.clone();
        thread::spawn(move ||{
            let client_arc = Arc::new(client);
            let client_arc2 = client_arc.clone();
            let rx2 = rx.clone();
            thread::spawn(move ||{
                client_arc2.run(backend, playback_name);
            });
            thread::spawn(move||{
                let c_rx = rx2.lock().unwrap();
                loop{
                    //TODO: Implement some queue/buffer/idk
                    let packet = c_rx.recv();
                    if packet.is_err(){
                        continue;
                    }
                    let p = packet.unwrap();
                    client_arc.send_opus(p);
                }
            });
        });
    });

    app.run().unwrap();
    
    
}
