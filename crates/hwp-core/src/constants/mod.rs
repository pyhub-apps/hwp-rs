pub mod ctrl_id;
pub mod fill_type;
pub mod tag_id;

pub const HWP_SIGNATURE: &[u8] =
    b"HWP Document File\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
pub const HWP_SIGNATURE_LEN: usize = 32;

/// HWP version struct
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HwpVersion {
    pub major: u8,
    pub minor: u8,
    pub build: u8,
    pub revision: u8,
}

impl HwpVersion {
    pub fn new(major: u8, minor: u8, build: u8, revision: u8) -> Self {
        Self {
            major,
            minor,
            build,
            revision,
        }
    }

    pub fn from_u32(value: u32) -> Self {
        Self {
            major: ((value >> 24) & 0xFF) as u8,
            minor: ((value >> 16) & 0xFF) as u8,
            build: ((value >> 8) & 0xFF) as u8,
            revision: (value & 0xFF) as u8,
        }
    }

    pub fn to_u32(&self) -> u32 {
        ((self.major as u32) << 24)
            | ((self.minor as u32) << 16)
            | ((self.build as u32) << 8)
            | (self.revision as u32)
    }

    pub fn is_supported(&self) -> bool {
        // Support HWP 5.0.0.0 and above
        self.major >= 5
    }
}

impl std::fmt::Display for HwpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.major, self.minor, self.build, self.revision
        )
    }
}
