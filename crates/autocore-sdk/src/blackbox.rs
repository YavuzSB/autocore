//! autocore-sdk :: Siyah Kutu (Blackbox) Teşhis ve Uçuş Verisi Kaydedicisi
//!
//! Dairesel ikili bellek tamponu (Ring Buffer Binary Logger) mimarisinde çalışır.
//! Her karede State (64B) + Command (32B) + Feedback (72B) + Zaman Damgası (8B) + CRC-16 (2B)
//! olmak üzere 184 baytlık sabit C-ABI blokları kaydeder.
//! Tampon dolduğunda en eski kayıtlar atomik olarak ezilir.

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use autocore_base::{ActuatorFeedback, KinematicCommand, StateVector6DoF};
use autocore_hal::protocol::crc16_ccitt;
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use static_assertions::{assert_eq_align, assert_eq_size};
use thiserror::Error;

pub const BLACKBOX_MAGIC: [u8; 4] = *b"BBOX";
pub const BLACKBOX_VERSION: u32 = 1;

#[derive(Error, Debug)]
pub enum BlackboxError {
    #[error("Geçersiz sihirli bayt dizisi (Magic bytes): beklenen BBOX")]
    InvalidMagic,
    #[error("Desteklenmeyen blackbox sürümü: {0}")]
    UnsupportedVersion(u32),
    #[error("Bozuk veya yetersiz başlık uzunluğu")]
    InvalidHeaderSize,
    #[error("Bozuk çerçeve boyutu: beklenen {0}, okunan {1}")]
    MismatchedFrameSize(u32, usize),
    #[error("Çerçeve CRC-16 sağlama toplamı hatası indeks {0}: beklenen {1:#06x}, okunan {2:#06x}")]
    CrcMismatch(usize, u16, u16),
    #[error("G/Ç Hatası: {0}")]
    Io(#[from] std::io::Error),
}

/// Siyah Kutu Dosya Başlığı (40 Bayt, C-ABI)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, Serialize, Deserialize)]
pub struct BlackboxHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub capacity: u32,
    pub record_size: u32,
    pub total_records: u64,
    pub start_time_us: u64,
    pub end_time_us: u64,
}

assert_eq_size!(BlackboxHeader, [u8; 40]);
assert_eq_align!(BlackboxHeader, u64);

/// Siyah Kutu Telemetri Çerçevesi (184 Bayt, C-ABI)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct BlackboxFrame {
    pub timestamp_us: u64,
    pub state: StateVector6DoF,
    pub command: KinematicCommand,
    pub feedback: ActuatorFeedback,
    pub crc16: u16,
    pub _padding: [u8; 6],
}

assert_eq_size!(BlackboxFrame, [u8; 184]);
assert_eq_align!(BlackboxFrame, u64);

impl BlackboxFrame {
    /// Yeni bir çerçeve oluşturur ve ilk 176 bayt üzerinden CRC-16-CCITT hesaplar
    pub fn new(
        timestamp_us: u64,
        state: StateVector6DoF,
        command: KinematicCommand,
        feedback: ActuatorFeedback,
    ) -> Self {
        let mut frame = Self {
            timestamp_us,
            state,
            command,
            feedback,
            crc16: 0,
            _padding: [0; 6],
        };
        let payload = &bytemuck::bytes_of(&frame)[..176];
        frame.crc16 = crc16_ccitt(payload);
        frame
    }

    /// Çerçevenin CRC-16 doğruluğunu denetler
    pub fn verify_crc(&self) -> bool {
        let payload = &bytemuck::bytes_of(self)[..176];
        let computed = crc16_ccitt(payload);
        self.crc16 == computed
    }
}

/// Dairesel İkili Halka Tamponu (Ring Buffer Recorder)
pub struct BlackboxRecorder {
    capacity: usize,
    buffer: Vec<BlackboxFrame>,
    head: usize,
    total_records: u64,
    is_full: bool,
}

impl BlackboxRecorder {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Blackbox kapasitesi 0 olamaz");
        Self {
            capacity,
            buffer: vec![bytemuck::Zeroable::zeroed(); capacity],
            head: 0,
            total_records: 0,
            is_full: false,
        }
    }

    /// Yeni bir telemetri anlık görüntüsü kaydeder
    pub fn record(
        &mut self,
        timestamp_us: u64,
        state: StateVector6DoF,
        command: KinematicCommand,
        feedback: ActuatorFeedback,
    ) {
        let frame = BlackboxFrame::new(timestamp_us, state, command, feedback);
        self.buffer[self.head] = frame;
        self.head = (self.head + 1) % self.capacity;
        self.total_records += 1;
        if self.head == 0 {
            self.is_full = true;
        }
    }

    /// Kaydedilen çerçeveleri en eskiden en yeniye kronolojik sırada döndürür
    pub fn ordered_frames(&self) -> Vec<BlackboxFrame> {
        if !self.is_full {
            self.buffer[..self.head].to_vec()
        } else {
            let mut result = Vec::with_capacity(self.capacity);
            result.extend_from_slice(&self.buffer[self.head..]);
            result.extend_from_slice(&self.buffer[..self.head]);
            result
        }
    }

    /// Başlık ve çerçeveleri ikili dosya olarak kaydeder
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), BlackboxError> {
        let frames = self.ordered_frames();
        let (start_time, end_time) = if frames.is_empty() {
            (0, 0)
        } else {
            (frames.first().unwrap().timestamp_us, frames.last().unwrap().timestamp_us)
        };

        let header = BlackboxHeader {
            magic: BLACKBOX_MAGIC,
            version: BLACKBOX_VERSION,
            capacity: self.capacity as u32,
            record_size: std::mem::size_of::<BlackboxFrame>() as u32,
            total_records: self.total_records,
            start_time_us: start_time,
            end_time_us: end_time,
        };

        let mut file = File::create(path)?;
        file.write_all(bytemuck::bytes_of(&header))?;
        for frame in &frames {
            file.write_all(bytemuck::bytes_of(frame))?;
        }
        file.flush()?;
        Ok(())
    }

    /// CSV formatına dışa aktarır (Foxglove Studio veya Pandas için)
    pub fn export_csv(&self) -> String {
        let frames = self.ordered_frames();
        let mut out = String::from(
            "timestamp_us,pos_x,pos_y,pos_z,vel_x,vel_y,vel_z,cmd_vx,cmd_vy,cmd_vz,cmd_yaw_rate,fb_curr0,fb_curr1,fb_temp0,fb_temp1\n"
        );
        for f in &frames {
            out.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                f.timestamp_us,
                f.state.position[0], f.state.position[1], f.state.position[2],
                f.state.velocity[0], f.state.velocity[1], f.state.velocity[2],
                f.command.linear[0], f.command.linear[1], f.command.linear[2],
                f.command.angular[2],
                f.feedback.motor_currents[0], f.feedback.motor_currents[1],
                f.feedback.temperatures[0], f.feedback.temperatures[1]
            ));
        }
        out
    }

    /// JSON formatına dışa aktarır
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let frames = self.ordered_frames();
        serde_json::to_string_pretty(&frames)
    }
}

/// İkili Siyah Kutu Dosyası Okuyucusu
pub struct BlackboxReader;

impl BlackboxReader {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<(BlackboxHeader, Vec<BlackboxFrame>), BlackboxError> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(BlackboxHeader, Vec<BlackboxFrame>), BlackboxError> {
        let header_size = std::mem::size_of::<BlackboxHeader>();
        if bytes.len() < header_size {
            return Err(BlackboxError::InvalidHeaderSize);
        }

        let header: BlackboxHeader = *bytemuck::try_from_bytes(&bytes[..header_size])
            .map_err(|_| BlackboxError::InvalidHeaderSize)?;

        if header.magic != BLACKBOX_MAGIC {
            return Err(BlackboxError::InvalidMagic);
        }

        if header.version != BLACKBOX_VERSION {
            return Err(BlackboxError::UnsupportedVersion(header.version));
        }

        let frame_size = std::mem::size_of::<BlackboxFrame>();
        let frame_bytes = &bytes[header_size..];

        if frame_bytes.len() % frame_size != 0 {
            return Err(BlackboxError::MismatchedFrameSize(frame_size as u32, frame_bytes.len()));
        }

        let num_frames = frame_bytes.len() / frame_size;
        let mut frames = Vec::with_capacity(num_frames);

        for i in 0..num_frames {
            let chunk = &frame_bytes[i * frame_size..(i + 1) * frame_size];
            let frame: BlackboxFrame = *bytemuck::try_from_bytes(chunk)
                .map_err(|_| BlackboxError::MismatchedFrameSize(frame_size as u32, chunk.len()))?;

            if !frame.verify_crc() {
                return Err(BlackboxError::CrcMismatch(i, frame.crc16, crc16_ccitt(&chunk[..176])));
            }
            frames.push(frame);
        }

        Ok((header, frames))
    }

    /// Kaza veya ani güç kesintisi durumunda dosyanın sonundaki yarım kalmış baytları
    /// izole ederek sağlam tüm çerçeveleri kurtarır (MIL-STD Uçuş Kaydedici Kurtarma)
    pub fn load_with_recovery<P: AsRef<Path>>(
        path: P,
    ) -> Result<(BlackboxHeader, Vec<BlackboxFrame>, usize), BlackboxError> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Self::from_bytes_with_recovery(&bytes)
    }

    pub fn from_bytes_with_recovery(
        bytes: &[u8],
    ) -> Result<(BlackboxHeader, Vec<BlackboxFrame>, usize), BlackboxError> {
        let header_size = std::mem::size_of::<BlackboxHeader>();
        if bytes.len() < header_size {
            return Err(BlackboxError::InvalidHeaderSize);
        }

        let header: BlackboxHeader = *bytemuck::try_from_bytes(&bytes[..header_size])
            .map_err(|_| BlackboxError::InvalidHeaderSize)?;

        if header.magic != BLACKBOX_MAGIC {
            return Err(BlackboxError::InvalidMagic);
        }

        if header.version != BLACKBOX_VERSION {
            return Err(BlackboxError::UnsupportedVersion(header.version));
        }

        let frame_size = std::mem::size_of::<BlackboxFrame>();
        let frame_bytes = &bytes[header_size..];

        let num_complete_frames = frame_bytes.len() / frame_size;
        let discarded_bytes = frame_bytes.len() % frame_size;

        let mut frames = Vec::with_capacity(num_complete_frames);

        for i in 0..num_complete_frames {
            let chunk = &frame_bytes[i * frame_size..(i + 1) * frame_size];
            let frame: BlackboxFrame = *bytemuck::try_from_bytes(chunk)
                .map_err(|_| BlackboxError::MismatchedFrameSize(frame_size as u32, chunk.len()))?;

            if !frame.verify_crc() {
                return Err(BlackboxError::CrcMismatch(i, frame.crc16, crc16_ccitt(&chunk[..176])));
            }
            frames.push(frame);
        }

        Ok((header, frames, discarded_bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blackbox_frame_crc_and_alignment() {
        let state = StateVector6DoF {
            position: [1.0, 2.0, -3.0],
            velocity: [5.0, 0.0, 0.0],
            ..Default::default()
        };
        let cmd = KinematicCommand {
            linear: [5.0, 0.0, 0.0],
            angular: [0.0, 0.0, 0.1],
            timestamp_us: 1000,
        };
        let fb = ActuatorFeedback {
            motor_currents: [2.5, 2.5, 0.0, 0.0],
            temperatures: [45.0, 46.0, 25.0, 25.0],
            ..Default::default()
        };

        let frame = BlackboxFrame::new(1000, state, cmd, fb);
        assert!(frame.verify_crc());

        // Bozuk bayt simülasyonu
        let mut corrupted = frame;
        corrupted.state.position[0] += 0.5;
        assert!(!corrupted.verify_crc());
    }

    #[test]
    fn test_blackbox_ring_buffer_wrap_and_readback() {
        let mut recorder = BlackboxRecorder::new(3);

        for i in 1..=5 {
            let mut state = StateVector6DoF::default();
            state.position[0] = i as f32;
            let cmd = KinematicCommand::default();
            let fb = ActuatorFeedback::default();
            recorder.record(i * 1000, state, cmd, fb);
        }

        // Kapasite 3; son 3 kayıt (3, 4, 5) kalmalı
        let frames = recorder.ordered_frames();
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].state.position[0], 3.0);
        assert_eq!(frames[1].state.position[0], 4.0);
        assert_eq!(frames[2].state.position[0], 5.0);

        // Geçici dosyaya kaydet ve geri oku
        let dir = tempfile::tempdir().unwrap();
        let log_path = dir.path().join("mission_flight.bbox");

        recorder.save_to_file(&log_path).unwrap();

        let (header, read_frames) = BlackboxReader::load_from_file(&log_path).unwrap();
        assert_eq!(header.magic, BLACKBOX_MAGIC);
        assert_eq!(header.total_records, 5);
        assert_eq!(header.capacity, 3);
        assert_eq!(read_frames.len(), 3);
        assert_eq!(read_frames[0].timestamp_us, 3000);
        assert_eq!(read_frames[2].timestamp_us, 5000);

        // CSV dışa aktarımı doğrula
        let csv = recorder.export_csv();
        assert!(csv.contains("timestamp_us,pos_x"));
        assert!(csv.contains("3000,3,0,0"));
    }
}
