// SPDX-FileCopyrightText: Copyright 2023 Savi
// SPDX-License-Identifier: GPL-3.0-only 

use slint::Model;
use std::rc::Rc;

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

pub fn main() {
    let app = App::new().unwrap();

    app.global::<PeerList>().on_change_volume(move |id, vol| {
        println!("{id}: {vol}%");
    });
    
    app.run().unwrap();
}
