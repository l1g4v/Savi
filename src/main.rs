// SPDX-FileCopyrightText: Copyright 2023 Savi
// SPDX-License-Identifier: GPL-3.0-only 

#[macro_use]
extern crate log;

use slint::Model;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicI32};
use std::sync::{Arc, Mutex};
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
    env_logger::init();
    let app = App::new().unwrap();
    let app_clone = app.clone_strong();
    let app_clone2 = app.clone_strong();
    let app_clone3 = app.clone_strong();
    let app_weak = app.as_weak();

    let playback_id = Arc::new(Mutex::new(String::new()));
    let playback_id_clone = playback_id.clone();
    let playback_id_clone2 = playback_id.clone();
    let playback_id_clone3 = playback_id.clone();
    let playback_id_clone4 = playback_id.clone();
    let capture_devices = Audio::get_input_devices();
    let playback_devices = Audio::get_output_devices();

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

    app.global::<AudioDevices>().set_capture_devices(crc.into());
    app.global::<AudioDevices>().set_playback_devices(prc.into());

    *playback_id_clone.lock().unwrap() = playback_devices[0].0.clone();

    let capture_device: Arc<Mutex<AudioCapture>> = Arc::new(Mutex::new(AudioCapture::new(capture_devices[0].1.clone(), 
    2, 48_000, 96_000, 0)));
    capture_device.lock().unwrap().start();
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
        *capture_device_clone.lock().unwrap() = AudioCapture::new(capture_devices[id as usize].1.clone(), 
        2, 48_000, 96_000, threshold);
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

    //Network
    let cs_instance: Arc<Mutex<(Option<SignalingClient>,Option<SignalingServer>)>> = Arc::new(Mutex::new((None, None)));
    let cs_instance_clone = cs_instance.clone();
    let cs_instance_clone2 = cs_instance.clone();

    app.global::<Signaling>().on_create(move ||{
        let username = app_clone2.global::<SelfPeer>().get_name().to_string();
        
        let server = SignalingServer::new(username);
        let listen = server.get_listen_address();
        let key = server.get_cipher_key();
        cs_instance_clone.lock().unwrap().1 = Some(server);

        let playback_name = playback_id_clone3.lock().unwrap().clone();
        let cs_sinstance = cs_instance_clone.clone();
        thread::spawn(move ||{
            let t = cs_sinstance.lock().unwrap();
            let server = t.1.as_ref().unwrap(); 
            server.run(playback_name);
        });
        
        app_clone2.global::<Signaling>().set_address(slint::SharedString::from(listen));
        app_clone2.global::<Signaling>().set_key(slint::SharedString::from(key));
        app_clone2.global::<Signaling>().set_hosting(true);
        
        let cd = capture_device_clone4.clone();
        let cs_sisntance2 = cs_instance_clone.clone();
        thread::spawn(move ||{
            let socket = cd.lock().unwrap().connect2queue();
            let buf = &mut [0; 1024];
            loop {
                let (amt, _src) = socket.recv_from(buf).unwrap();
                if amt == 0 {
                    continue;
                }
                let packet = buf[..amt].to_vec().clone();
                cs_sisntance2.lock().unwrap().1.as_ref().unwrap().send_opus(packet);
            }
            /*
            let t = cs_sisntance2.lock().unwrap();
            let server = t.1.as_ref().unwrap(); 

            let capture_arc = cd.clone().lock().unwrap().get_capture_arc();
            let (mutex, cvar) = &*capture_arc;
            mutex.lock().unwrap().clear();
            loop {
                let mut queue = mutex.lock().unwrap();
                while queue.is_empty() {
                    queue = cvar.wait(queue).unwrap(); // Wait until the vector has elements
                }    
                while queue.len() > 0{
                    debug!("Sending opus packet");
                    let payload = queue.pop().unwrap();
                    server.send_opus(payload);
                }
            }*/
        });

    });
    //TODO: implement a socket to read from AudioCapture
    app.global::<Signaling>().on_connect(move |addr, key|{
        let username = app_clone3.global::<SelfPeer>().get_name().to_string();

        let client = SignalingClient::new(username, addr.to_string(), key.to_string());
        cs_instance_clone2.lock().unwrap().0 = Some(client);
        let playback_name = playback_id_clone4.lock().unwrap().clone();
        let cs_cinstance = cs_instance_clone2.clone();
        thread::spawn(move ||{
            let t = cs_cinstance.lock().unwrap();
            let client = t.0.as_ref().unwrap();
            client.run(playback_name);
        });

        

        println!("Connecting to {}", addr.as_str());
        println!("Key: {}", key.as_str());
        app_clone3.global::<Signaling>().set_connected(true);

        let cd = capture_device_clone5.clone();
        let cs_cinstance2 = cs_instance_clone2.clone();
        thread::spawn(move ||{
            let socket = cd.lock().unwrap().connect2queue();
            let buf = &mut [0; 1024];
            loop {
                let (amt, _src) = socket.recv_from(buf).unwrap();
                if amt == 0 {
                    continue;
                }
                let packet = buf[..amt].to_vec().clone();
                cs_cinstance2.lock().unwrap().0.as_ref().unwrap().send_opus(packet);
            }
            /*
            let t = cs_cinstance2.lock().unwrap();
            let client = t.0.as_ref().unwrap();

            let capture_arc = cd.clone().lock().unwrap().get_capture_arc();
            let (mutex, cvar) = &*capture_arc;
            mutex.lock().unwrap().clear();
            loop {
                let mut queue = mutex.lock().unwrap();
                while queue.is_empty() {
                    queue = cvar.wait(queue).unwrap(); // Wait until the vector has elements
                }    
                while queue.len() > 0{
                    debug!("Sending opus packet");
                    let payload = queue.pop().unwrap();
                    client.send_opus(payload);
                }
            }*/
        });
    });

    app.run().unwrap();
    
    
}
