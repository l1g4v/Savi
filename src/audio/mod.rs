// SPDX-FileCopyrightText: Copyright 2023 Savi
// SPDX-License-Identifier: GPL-3.0-only 

use miniaudio::{Context, DeviceId, DeviceIdAndName};

pub mod capture;
pub mod playback;

#[derive(PartialEq)]
pub enum DeviceKind{
    Capture,
    Playback,
}
pub struct Audio {}
impl Audio {
    /// Returns all the capture devices
    pub fn get_input_devices() -> Vec<(String, DeviceId)> {
        let context = Context::new(&[], None).unwrap();
        let mut inputs: Vec<(String, DeviceId)> = Vec::new();
        context
            .with_devices(|_, capture_devices| {
                for (_, device) in capture_devices.iter().enumerate() {
                    inputs.push((device.name().to_string(), device.id().clone()));
                }
            })
            .expect("failed to get devices");
        inputs
    }

    /// Returns all the playback devices
    pub fn get_output_devices() -> Vec<(String, DeviceId)> {
        let context = Context::new(&[], None).unwrap();
        let mut outputs: Vec<(String, DeviceId)> = Vec::new();
        context
            .with_devices(|playback_devices, _| {
                for (_, device) in playback_devices.iter().enumerate() {
                    outputs.push((device.name().to_string(), device.id().clone()));
                }
            })
            .expect("failed to get devices");
        outputs
    }

    /// Prints all the capture and playback devices (used for debugging)
    pub fn print_devices() {
        let context = Context::new(&[], None).unwrap();

        context
            .with_devices(|playback_devices, capture_devices| {
                println!("Playback Devices:");
                for (idx, device) in playback_devices.iter().enumerate() {
                    println!("\t{}: {}", idx, device.name());
                }

                println!("Capture Devices:");
                for (idx, device) in capture_devices.iter().enumerate() {
                    println!("\t{}: {}", idx, device.name());
                }
            })
            .expect("failed to get devices");
    }

    pub fn get_device_id(name: &String, kind: DeviceKind) -> Option<DeviceId>{
        let context = Context::new(&[], None).unwrap();
        let mut id = None;
        context
            .with_devices(|playback_devices, capture_devices| {
                if kind == DeviceKind::Capture{
                    for device in capture_devices.iter() {
                        if device.name() == name {
                            id = Some(device.id().clone());
                        }
                    }
                }
                else{
                    for device in playback_devices.iter() {
                        if device.name() == name {
                            id = Some(device.id().clone());
                        }
                    }
                }
                
            })
            .expect("failed to get devices");
        id
    }
}
