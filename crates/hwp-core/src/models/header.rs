use crate::constants::HwpVersion;

fn default_reserved() -> [u8; 216] {
    [0u8; 216]
}

/// HWP file header structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HwpHeader {
    /// File signature (32 bytes)
    #[cfg_attr(feature = "serde", serde(with = "serde_arrays"))]
    pub signature: [u8; 32],
    
    /// HWP version
    pub version: HwpVersion,
    
    /// File properties (bit flags)
    pub properties: HwpProperties,
    
    /// Reserved bytes
    #[cfg_attr(feature = "serde", serde(skip, default = "default_reserved"))]
    pub reserved: [u8; 216],
}

impl HwpHeader {
    pub const SIZE: usize = 256; // Total header size in bytes
    
    /// Check if the document is compressed
    pub fn is_compressed(&self) -> bool {
        self.properties.compressed
    }
    
    /// Check if the document has a password
    pub fn has_password(&self) -> bool {
        self.properties.has_password
    }
    
    /// Check if the document is DRM protected
    pub fn is_drm_document(&self) -> bool {
        self.properties.is_drm_document
    }
}

/// HWP file properties (from bit flags)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HwpProperties {
    pub compressed: bool,
    pub has_password: bool,
    pub is_distribution_document: bool,
    pub has_script: bool,
    pub is_drm_document: bool,
    pub has_xml_template_storage: bool,
    pub has_document_history: bool,
    pub has_certificate_signature: bool,
    pub certificate_encryption: bool,
    pub has_certificate_drm: bool,
    pub is_ccl_document: bool,
    pub is_mobile_optimized: bool,
    pub is_private_information_security: bool,
    pub has_change_tracking: bool,
    pub has_kogl_copyright: bool,
    pub has_video_control: bool,
    pub has_order_field_control: bool,
    pub reserved_bits: u16, // For future use
}

impl HwpProperties {
    /// Create properties from a 32-bit value
    pub fn from_u32(value: u32) -> Self {
        Self {
            compressed: (value & 0x0001) != 0,
            has_password: (value & 0x0002) != 0,
            is_distribution_document: (value & 0x0004) != 0,
            has_script: (value & 0x0008) != 0,
            is_drm_document: (value & 0x0010) != 0,
            has_xml_template_storage: (value & 0x0020) != 0,
            has_document_history: (value & 0x0040) != 0,
            has_certificate_signature: (value & 0x0080) != 0,
            certificate_encryption: (value & 0x0100) != 0,
            has_certificate_drm: (value & 0x0200) != 0,
            is_ccl_document: (value & 0x0400) != 0,
            is_mobile_optimized: (value & 0x0800) != 0,
            is_private_information_security: (value & 0x1000) != 0,
            has_change_tracking: (value & 0x2000) != 0,
            has_kogl_copyright: (value & 0x4000) != 0,
            has_video_control: (value & 0x8000) != 0,
            has_order_field_control: (value & 0x10000) != 0,
            reserved_bits: ((value >> 17) & 0x7FFF) as u16,
        }
    }
    
    /// Convert properties to a 32-bit value
    pub fn to_u32(&self) -> u32 {
        let mut value = 0u32;
        if self.compressed { value |= 0x0001; }
        if self.has_password { value |= 0x0002; }
        if self.is_distribution_document { value |= 0x0004; }
        if self.has_script { value |= 0x0008; }
        if self.is_drm_document { value |= 0x0010; }
        if self.has_xml_template_storage { value |= 0x0020; }
        if self.has_document_history { value |= 0x0040; }
        if self.has_certificate_signature { value |= 0x0080; }
        if self.certificate_encryption { value |= 0x0100; }
        if self.has_certificate_drm { value |= 0x0200; }
        if self.is_ccl_document { value |= 0x0400; }
        if self.is_mobile_optimized { value |= 0x0800; }
        if self.is_private_information_security { value |= 0x1000; }
        if self.has_change_tracking { value |= 0x2000; }
        if self.has_kogl_copyright { value |= 0x4000; }
        if self.has_video_control { value |= 0x8000; }
        if self.has_order_field_control { value |= 0x10000; }
        value |= (self.reserved_bits as u32) << 17;
        value
    }
}