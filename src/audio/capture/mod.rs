// SPDX-FileCopyrightText: Copyright 2023 Savi
// SPDX-License-Identifier: GPL-3.0-only 
use miniaudio::{Device, DeviceId, Format, ShareMode, DeviceConfig, DeviceType};
use std::{sync::{Arc, Mutex, Condvar, atomic::AtomicI32}};
use opus::{Encoder, Application, Channels, Bitrate};

pub struct AudioCapture{
    capture_arc: Arc<(Mutex<Vec<Vec<u8>>>, Condvar)>,
    capture_device: Device,
    intensity: Arc<AtomicI32>,
    threshold: Arc<AtomicI32>,
    encoder: Arc<Mutex<Encoder>>,
}
impl AudioCapture {
    /// Creates a new AudioCapture instance
    /// # Arguments
    /// * `device_id` - The DeviceId of the device to use
    /// * `channels` - The number of channels to use
    /// * `sample_rate` - The sample rate to use
    /// * `encoder_bitrate` - The bitrate to use for the encoder
    /// * `active_threshold` - The RMS threshold to record and encode the sample
    pub fn new(device_id: DeviceId, channels: u32, sample_rate: u32, encoder_bitrate: i32, active_threshold: i32) -> Self{
        let capture_arc = Arc::new((Mutex::new(Vec::new()), Condvar::new()));
        let capture_clone = capture_arc.clone();

        let intensity = Arc::new(AtomicI32::new(-100));
        let intensity_clone = intensity.clone();

        let threshold = Arc::new(AtomicI32::new(active_threshold));
        let threshold_clone = threshold.clone();

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
        let encoder = Arc::new(Mutex::new(Encoder::new(sample_rate, encoder_channels, Application::Voip).unwrap()));
        encoder.lock().unwrap().set_bitrate(Bitrate::Bits(encoder_bitrate)).unwrap();
        encoder.lock().unwrap().set_vbr(true).unwrap();
        let encoder_clone = encoder.clone();

        let mut capture_device: Device = Device::new(None, &config).unwrap();
        capture_device.set_data_callback(move |_, _, input|{
            let input_samples = input.as_samples::<i16>();
            let num_samples = input_samples.len();
            //let i16_max = i16::MAX as f32;
            //Calculate the sample RMS
            let sum: f32 = input_samples.iter().map(|&s| (s as f32 /i16::MAX as f32).powi(2)).sum();
            let rms = (((sum / num_samples as f32).sqrt() + 0.0002)*100.0) as i32;
            intensity_clone.store(rms, std::sync::atomic::Ordering::Relaxed);

            //If the RMS is above the threshold, encode and push to the queue
            if rms > threshold_clone.load(std::sync::atomic::Ordering::Relaxed){
                let (vec, cvar) = &*capture_clone; 
                let encoded = encoder_clone.lock().unwrap().encode_vec(input_samples, num_samples).unwrap();
                
                //Push and notify to all threads waiting on content
                vec.lock().unwrap().push(encoded);
                cvar.notify_all();
            }
        });
        AudioCapture { capture_arc,  capture_device, intensity, threshold, encoder }
    }

    /// Starts the capture device
    pub fn start(&self){
        self.capture_device.start().unwrap();
    }

    /// Stops the capture device
    pub fn stop(&self){
        self.capture_device.stop().unwrap();
    }

    /// Returns the capture arc
    pub fn get_capture_arc(&self) -> Arc<(Mutex<Vec<Vec<u8>>>, Condvar)>{
        self.capture_arc.clone()
    }

    /// Returns the intensity
    pub fn get_intensity(&self) -> i32{
        self.intensity.clone().load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Changes the threshold
    pub fn set_threshold(&self, value: i32){
        self.threshold.store(value, std::sync::atomic::Ordering::Relaxed);
    }

    /// Changes the encoder bitrate
    pub fn set_encoder_bitrate(&self, value: i32){
        self.encoder.lock().unwrap().set_bitrate(Bitrate::Bits(value)).unwrap();
    }


}