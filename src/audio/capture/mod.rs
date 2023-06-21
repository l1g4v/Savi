use miniaudio::{Device, DeviceId, Format, ShareMode, DeviceConfig, DeviceType};
use std::{sync::{Arc, Mutex, Condvar}};
use opus::{Encoder, Application, Channels, Bitrate};

pub struct AudioCapture{
    capture_arc: Arc<(Mutex<Vec<Vec<u8>>>, Condvar)>,
    capture_device: Device,
}
impl AudioCapture {
    /// Creates a new AudioCapture instance
    /// # Arguments
    /// * `device_id` - The DeviceId of the device to use
    /// * `channels` - The number of channels to use
    /// * `sample_rate` - The sample rate to use
    /// * `encoder_bitrate` - The bitrate to use for the encoder
    /// * `active_threshold` - The RMS threshold to record and encode the sample
    pub fn new(device_id: DeviceId, channels: u32, sample_rate: u32, encoder_bitrate: i32, active_threshold: f32) -> Self{
        let capture_arc = Arc::new((Mutex::new(Vec::new()), Condvar::new()));
        let capture_clone = capture_arc.clone();

        let mut config = DeviceConfig::new(DeviceType::Capture);
        config.capture_mut().set_format(Format::S16);
        config.capture_mut().set_channels(channels);
        config.capture_mut().set_share_mode(ShareMode::Shared);
        config.capture_mut().set_device_id(Some(device_id));
        config.set_sample_rate(sample_rate);

        
        let encoder_channels = match channels {
            1 => Channels::Mono,
            2 => Channels::Stereo,
            _ => panic!("Invalid channel count"),
        };
        let mut encoder = Encoder::new(sample_rate, encoder_channels, Application::Voip).unwrap();
        encoder.set_bitrate(Bitrate::Bits(encoder_bitrate)).unwrap();
        encoder.set_vbr(true).unwrap();

        let mut capture_device: Device = Device::new(None, &config).unwrap();
        capture_device.set_data_callback(move |_, _, input|{
            let input_samples = input.as_samples::<i16>();
            let num_samples = input_samples.len();

            //Calculate the sample RMS
            let sum: f32 = input_samples.iter().map(|&s| (s as f32 * s as f32)).sum();
            let rms = (sum / num_samples as f32).sqrt();//.sqrt();

            //If the RMS is above the threshold, encode and push to the queue
            if rms > active_threshold{
                let (vec, cvar) = &*capture_clone; 
                let encoded = encoder.encode_vec(input_samples, num_samples).unwrap();
                
                //Push and notify to all threads waiting on content
                vec.lock().unwrap().push(encoded);
                cvar.notify_all();
            }
        });
        AudioCapture { capture_arc,  capture_device }
    }

    /// Starts the capture device
    pub fn start(&self){
        self.capture_device.start().unwrap();
    }

    /// Returns the capture arc
    pub fn get_capture_arc(&self) -> Arc<(Mutex<Vec<Vec<u8>>>, Condvar)>{
        self.capture_arc.clone()
    }
}