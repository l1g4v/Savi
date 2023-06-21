use miniaudio::{Device, DeviceId, Format, ShareMode, DeviceConfig, DeviceType};
use std::{sync::{Arc, Mutex, Condvar}};
use opus::{Decoder, Channels};

pub struct AudioPlayback{
    playback_arc: Arc<(Mutex<Vec<Vec<u8>>>, Condvar)>,
    playback_device: Device,
}
impl AudioPlayback {

    /// Creates a DeviceConfig for a playback device
    /// # Arguments
    /// * `device_id` - The DeviceId of the device to use
    /// * `channels` - The number of channels to use
    /// * `sample_rate` - The sample rate to use
    pub fn create_config(device_id: DeviceId, channels: u32, sample_rate: u32) -> DeviceConfig{
        let mut config = DeviceConfig::new(DeviceType::Playback);
        config.playback_mut().set_format(Format::S16);
        config.playback_mut().set_channels(channels);
        config.playback_mut().set_share_mode(ShareMode::Shared);
        config.playback_mut().set_device_id(Some(device_id));
        config.set_sample_rate(sample_rate);
        config
    }

    /// Creates a new AudioPlayback instance
    /// # Arguments
    /// * `config` - The DeviceConfig to use
    pub fn new(config: DeviceConfig) -> Self{
        let playback_arc = Arc::new((Mutex::new(Vec::<Vec<u8>>::new()), Condvar::new()));
        let playback_clone = playback_arc.clone();
        
        let decoder_channels = match config.playback().channels() {
            1 => Channels::Mono,
            2 => Channels::Stereo,
            _ => panic!("Invalid channel count"),
        };
        let mut decoder = Decoder::new(config.sample_rate(), decoder_channels).unwrap();

        let mut playback_device: Device = Device::new(None, &config).unwrap();
        playback_device.set_data_callback(move |_, output, _|{ 
            let (mutex, cvar) = &*playback_clone;
            let mut queue = mutex.lock().unwrap();
            //Hold the thread until there is content in the queue (avoids absurd CPU usage)
            //TODO[playback]: Find a better way to do this (I'm still not sure if blocking the miniaudio data callback is a good idea)
            while queue.is_empty(){
                queue = cvar.wait(queue).unwrap();
            }

            let len = output.as_samples_mut::<i16>().len();
            let mut decoded: Vec<i16> = vec![0; len*2];
            if queue.len() > 1 {
                //Decode opus packet
                let payload = queue.remove(0);
                let _ = decoder.decode(&payload.as_slice()[..payload.len()-1], decoded.as_mut_slice(), false).unwrap();

                //Apply volume by scaling the decoded samples
                let volume = payload[payload.len()-1] as f32 / 100.0;
                decoded.iter_mut().for_each(|x| *x = (*x as f32 * volume)as i16);

                //Copy the decoded samples to the output buffer
                output.as_samples_mut::<i16>().copy_from_slice(&decoded.as_slice()[..len]);
            }
        });
        AudioPlayback { playback_arc,  playback_device }
    }

    /// Starts the playback device
    pub fn start(&self){
        self.playback_device.start().unwrap();
    }

    /// Returns a clone of the playback queue
    pub fn get_playback_arc(&self) -> Arc<(Mutex<Vec<Vec<u8>>>, Condvar)>{
        self.playback_arc.clone()
    }
}