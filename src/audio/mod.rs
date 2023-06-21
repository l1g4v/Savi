use miniaudio::{Context, DeviceId};

pub mod capture;
pub mod playback;

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
}
