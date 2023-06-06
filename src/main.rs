// SPDX-FileCopyrightText: Copyright 2023 Savi
// SPDX-License-Identifier: GPL-3.0-only 

#[macro_use]
extern crate log;

use slint::Model;
use std::rc::Rc;
use tokio::runtime::Runtime;
mod aes;
mod signaling_client;
mod signaling_server;
slint::include_modules!();


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

    app.global::<PeerList>().on_change_volume(move |id, vol| {
        println!("{id}: {vol}%");
    });
    
    app.run().unwrap();
}
