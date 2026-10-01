use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, SpatialSink, Source};
use std::fs::File;
use std::io::BufReader;
use nalgebra_glm::Vec3;

pub struct AudioManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    bgm_sink: Sink,
    pub portal_sink: SpatialSink,
    pub pig_sink: SpatialSink,
    current_realm: String,
}

impl AudioManager {
    pub fn new() -> Self {
        // Setup output stream
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        
        let bgm_sink = Sink::try_new(&stream_handle).unwrap();
        // Left ear at -1 on X axis, right ear at +1 on X axis
        let portal_sink = SpatialSink::try_new(&stream_handle, [0.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]).unwrap();
        let pig_sink = SpatialSink::try_new(&stream_handle, [0.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]).unwrap();
        
        // Loop portal sound continuously but start muted
        if let Ok(file) = File::open("assets/Nether_Portal.mp3") {
            if let Ok(decoder) = Decoder::new(BufReader::new(file)) {
                let source = decoder.repeat_infinite();
                portal_sink.append(source);
                portal_sink.play();
                portal_sink.set_volume(0.0);
            }
        }

        Self {
            _stream,
            stream_handle,
            bgm_sink,
            portal_sink,
            pig_sink,
            current_realm: String::new(),
        }
    }
    
    pub fn play_music(&mut self, realm: &str) {
        if self.current_realm == realm {
            return;
        }
        self.current_realm = realm.to_string();
        
        let filename = if realm == "Overworld" {
            "assets/Haunt_Muskie.mp3"
        } else {
            "assets/Dead_Voxel.mp3"
        };
        
        self.bgm_sink.stop(); // Stop current music
        self.bgm_sink = Sink::try_new(&self.stream_handle).unwrap(); // Recreate sink to clear queue
        
        if let Ok(file) = File::open(filename) {
            if let Ok(decoder) = Decoder::new(BufReader::new(file)) {
                let source = decoder.repeat_infinite();
                self.bgm_sink.append(source);
                self.bgm_sink.set_volume(0.4);
                self.bgm_sink.play();
            }
        }
    }
    
    pub fn update_3d_audio(&mut self, camera_pos: Vec3, camera_dir: Vec3, portal_pos: Vec3, pig_pos: Vec3, in_nether: bool) {
        let right_dir = camera_dir.cross(&Vec3::new(0.0, 1.0, 0.0)).normalize();
        
        // Portal audio
        let portal_vec = portal_pos - camera_pos;
        let portal_dist = portal_vec.norm();
        
        // Portal sound is louder and has larger radius in the Nether
        let max_dist = if in_nether { 25.0 } else { 15.0 };
        let portal_vol = if portal_dist < max_dist {
            1.0 - (portal_dist / max_dist)
        } else {
            0.0
        };
        
        self.portal_sink.set_volume(portal_vol * 0.5);
        
        // Project onto relative axes for spatial audio
        let px = portal_vec.dot(&right_dir);
        let py = portal_vec.dot(&Vec3::new(0.0, 1.0, 0.0));
        let pz = portal_vec.dot(&camera_dir);
        self.portal_sink.set_emitter_position([px, py, pz]);
        
        // Pig audio (Only in overworld, distance based)
        let pig_vec = pig_pos - camera_pos;
        let pig_dist = pig_vec.norm();
        let pig_vol = if !in_nether && pig_dist < 20.0 {
            1.0 - (pig_dist / 20.0)
        } else {
            0.0
        };
        self.pig_sink.set_volume(pig_vol * 1.5);
        
        let p_x = pig_vec.dot(&right_dir);
        let p_y = pig_vec.dot(&Vec3::new(0.0, 1.0, 0.0));
        let p_z = pig_vec.dot(&camera_dir);
        self.pig_sink.set_emitter_position([p_x, p_y, p_z]);
    }
    
    pub fn play_pig_sfx(&mut self) {
        if self.pig_sink.empty() {
            if let Ok(file) = File::open("assets/Pig_sfx.mp3") {
                if let Ok(decoder) = Decoder::new(BufReader::new(file)) {
                    self.pig_sink.append(decoder);
                    self.pig_sink.play();
                }
            }
        }
    }
}
