#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ColorSource {
    CcmuxCombined,
    CcmuxTexel0,
    CcmuxTexel1,
    CcmuxPrimitive,
    CcmuxShade,
    CcmuxEnvironment,
    Ccmux1,
    CcmuxNoise,
    Ccmux0,
    CcmuxCenter,
    CcmuxK4,
    CcmuxScale,
    CcmuxCombinedAlpha,
    CcmuxTexel0Alpha,
    CcmuxTexel1Alpha,
    CcmuxPrimitiveAlpha,
    CcmuxShadeAlpha,
    CcmuxEnvAlpha,
    CcmuxLodFraction,
    CcmuxPrimLodFrac,
    CcmuxK5,

    AcmuxCombined,
    AcmuxLodFraction,
    AcmuxTexel0,
    AcmuxTexel1,
    AcmuxPrimitive,
    AcmuxShade,
    AcmuxEnvironment,
    Acmux1,
    Acmux0,
    AcmuxPrimLodFrac,
}
impl ColorSource {
    fn decode_color_0_5(value: u8) -> Self {
        assert!(value <= 0x05);
        match value {
            0x00 => Self::CcmuxCombined,
            0x01 => Self::CcmuxTexel0,
            0x02 => Self::CcmuxTexel1,
            0x03 => Self::CcmuxPrimitive,
            0x04 => Self::CcmuxShade,
            0x05 => Self::CcmuxEnvironment,
            _ => unreachable!(),
        }
    }

    /// Decode a 4-bit _Color A_ value.
    pub fn decode_color_a(value: u8) -> Option<Self> {
        match value {
            0x00..=0x05 => Some(Self::decode_color_0_5(value)),
            0x06 => Some(Self::Ccmux1),
            0x07 => Some(Self::CcmuxNoise),
            0x08..=0x0F => Some(Self::Ccmux0),
            0x10..=0xFF => None,
        }
    }

    /// Decode a 4-bit _Color B_ value.
    pub fn decode_color_b(value: u8) -> Option<Self> {
        match value {
            0x00..=0x05 => Some(Self::decode_color_0_5(value)),
            0x06 => Some(Self::CcmuxCenter),
            0x07 => Some(Self::CcmuxK4),
            0x08..=0x0F => Some(Self::Ccmux0),
            0x10..=0xFF => None,
        }
    }

    /// Decode a 5-bit _Color C_ value.
    pub fn decode_color_c(value: u8) -> Option<Self> {
        match value {
            0x00..=0x05 => Some(Self::decode_color_0_5(value)),
            0x06 => Some(Self::CcmuxScale),
            0x07 => Some(Self::CcmuxCombinedAlpha),
            0x08 => Some(Self::CcmuxTexel0Alpha),
            0x09 => Some(Self::CcmuxTexel1Alpha),
            0x0A => Some(Self::CcmuxPrimitiveAlpha),
            0x0B => Some(Self::CcmuxShadeAlpha),
            0x0C => Some(Self::CcmuxEnvAlpha),
            0x0D => Some(Self::CcmuxLodFraction),
            0x0E => Some(Self::CcmuxPrimLodFrac),
            0x0F => Some(Self::CcmuxK5),
            0x10..=0x1F => Some(Self::Ccmux0),
            0x20..=0xFF => None,
        }
    }

    /// Decode a 3-bit _Color D_ value.
    pub fn decode_color_d(value: u8) -> Option<Self> {
        match value {
            0x00..=0x05 => Some(Self::decode_color_0_5(value)),
            0x06 => Some(Self::Ccmux1),
            0x07 => Some(Self::Ccmux0),
            0x08..=0xFF => None,
        }
    }

    fn decode_alpha_1_5(value: u8) -> Self {
        assert!(value >= 0x01 && value <= 0x05);
        match value {
            0x01 => Self::AcmuxTexel0,
            0x02 => Self::AcmuxTexel1,
            0x03 => Self::AcmuxPrimitive,
            0x04 => Self::AcmuxShade,
            0x05 => Self::AcmuxEnvironment,
            _ => unreachable!(),
        }
    }

    /// Decode a 3-bit _Alpha A_, _Alpha B_ or _Alpha D_ value.
    pub fn decode_alpha_a_b_d(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::AcmuxCombined),
            0x01..=0x05 => Some(Self::decode_alpha_1_5(value)),
            0x06 => Some(Self::Acmux1),
            0x07 => Some(Self::Acmux0),
            0x08..=0xFF => None,
        }
    }

    /// Decode a 3-bit _Alpha C_ value.
    pub fn decode_alpha_c(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::AcmuxLodFraction),
            0x01..=0x05 => Some(Self::decode_alpha_1_5(value)),
            0x06 => Some(Self::AcmuxPrimLodFrac),
            0x07 => Some(Self::Acmux0),
            0x08..=0xFF => None,
        }
    }

    fn encode_color_0_5(&self) -> Option<u8> {
        match self {
            Self::CcmuxCombined => Some(0x00),
            Self::CcmuxTexel0 => Some(0x01),
            Self::CcmuxTexel1 => Some(0x02),
            Self::CcmuxPrimitive => Some(0x03),
            Self::CcmuxShade => Some(0x04),
            Self::CcmuxEnvironment => Some(0x05),
            _ => None,
        }
    }

    pub fn encode_color_a(&self) -> Option<u8> {
        if let Some(b) = self.encode_color_0_5() {
            return Some(b);
        }

        match self {
            Self::Ccmux1 => Some(0x06),
            Self::CcmuxNoise => Some(0x07),
            Self::Ccmux0 => Some(0x08), // any of 0x08..=0x0F
            _ => None,
        }
    }

    pub fn encode_color_b(&self) -> Option<u8> {
        if let Some(b) = self.encode_color_0_5() {
            return Some(b);
        }

        match self {
            Self::CcmuxCenter => Some(0x06),
            Self::CcmuxK4 => Some(0x07),
            Self::Ccmux0 => Some(0x08), // any of 0x08..=0x0F
            _ => None,
        }
    }

    pub fn encode_color_c(&self) -> Option<u8> {
        if let Some(b) = self.encode_color_0_5() {
            return Some(b);
        }

        match self {
            Self::CcmuxScale => Some(0x06),
            Self::CcmuxCombinedAlpha => Some(0x07),
            Self::CcmuxTexel0Alpha => Some(0x08),
            Self::CcmuxTexel1Alpha => Some(0x09),
            Self::CcmuxPrimitiveAlpha => Some(0x0A),
            Self::CcmuxShadeAlpha => Some(0x0B),
            Self::CcmuxEnvAlpha => Some(0x0C),
            Self::CcmuxLodFraction => Some(0x0D),
            Self::CcmuxPrimLodFrac => Some(0x0E),
            Self::CcmuxK5 => Some(0x0F),
            Self::Ccmux0 => Some(0x10), // any of 0x10..=0x1F
            _ => None,
        }
    }

    pub fn encode_color_d(&self) -> Option<u8> {
        if let Some(b) = self.encode_color_0_5() {
            return Some(b);
        }

        match self {
            Self::Ccmux1 => Some(0x06),
            Self::Ccmux0 => Some(0x07),
            _ => None,
        }
    }

    fn encode_alpha_1_5(&self) -> Option<u8> {
        match self {
            Self::AcmuxTexel0 => Some(0x01),
            Self::AcmuxTexel1 => Some(0x02),
            Self::AcmuxPrimitive => Some(0x03),
            Self::AcmuxShade => Some(0x04),
            Self::AcmuxEnvironment => Some(0x05),
            _ => None,
        }
    }

    pub fn encode_alpha_a_b_d(&self) -> Option<u8> {
        if let Some(b) = self.encode_alpha_1_5() {
            return Some(b);
        }

        match self {
            Self::AcmuxCombined => Some(0x00),
            Self::Acmux1 => Some(0x06),
            Self::Acmux0 => Some(0x07),
            _ => None,
        }
    }

    pub fn encode_alpha_c(&self) -> Option<u8> {
        if let Some(b) = self.encode_alpha_1_5() {
            return Some(b);
        }

        match self {
            Self::AcmuxLodFraction => Some(0x00),
            Self::AcmuxPrimLodFrac => Some(0x06),
            Self::Acmux0 => Some(0x07),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ColorSource;

    #[test]
    fn test_color_source_roundtrip() {
        for byte in 0..=0xFF {
            if let Some(ca) = ColorSource::decode_color_a(byte) {
                if ca != ColorSource::Ccmux0 {
                    assert_eq!(ca.encode_color_a().unwrap(), byte);
                }
            }
            if let Some(cb) = ColorSource::decode_color_b(byte) {
                if cb != ColorSource::Ccmux0 {
                    assert_eq!(cb.encode_color_b().unwrap(), byte);
                }
            }
            if let Some(cc) = ColorSource::decode_color_c(byte) {
                if cc != ColorSource::Ccmux0 {
                    assert_eq!(cc.encode_color_c().unwrap(), byte);
                }
            }
            if let Some(cd) = ColorSource::decode_color_d(byte) {
                if cd != ColorSource::Ccmux0 {
                    assert_eq!(cd.encode_color_d().unwrap(), byte);
                }
            }
            if let Some(aa) = ColorSource::decode_alpha_a_b_d(byte) {
                assert_eq!(aa.encode_alpha_a_b_d().unwrap(), byte);
            }
            if let Some(ac) = ColorSource::decode_alpha_c(byte) {
                assert_eq!(ac.encode_alpha_c().unwrap(), byte);
            }
        }
    }
}
