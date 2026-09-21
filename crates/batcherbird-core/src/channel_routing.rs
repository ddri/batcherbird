//! Channel routing configuration for audio input devices.
//!
//! Supports Stereo (inputs 1 & 2), Mono Left (input 1), and Mono Right (input 2).

use serde::{Deserialize, Serialize};

/// Input channel routing selection for recording and monitoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum ChannelRouting {
    /// Record channels 1 & 2 as interleaved stereo audio (channels: 2).
    /// If hardware device has only 1 channel, gracefully falls back to mono.
    #[default]
    Stereo = 0,

    /// Record channel 1 only as true mono audio (channels: 1).
    /// Direct monitoring playthrough centers this signal into both stereo output channels.
    MonoLeft = 1,

    /// Record channel 2 only as true mono audio (channels: 1).
    /// If hardware device has only 1 channel, gracefully falls back to channel 1.
    /// Direct monitoring playthrough centers this signal into both stereo output channels.
    MonoRight = 2,
}

impl ChannelRouting {
    /// Convert to u8 representation for atomic storage.
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Convert from u8 representation stored in atomic variables.
    pub const fn from_u8(val: u8) -> Self {
        match val {
            1 => Self::MonoLeft,
            2 => Self::MonoRight,
            _ => Self::Stereo,
        }
    }

    /// Human-readable label for UI pickers and displays.
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Stereo => "Stereo (1+2)",
            Self::MonoLeft => "Mono In 1 (L)",
            Self::MonoRight => "Mono In 2 (R)",
        }
    }

    /// Resulting output channel count for recorded samples given the hardware's channel count.
    pub const fn output_channels(&self, hw_channels: u16) -> u16 {
        match self {
            Self::Stereo => {
                if hw_channels >= 2 {
                    2
                } else {
                    1
                }
            }
            Self::MonoLeft | Self::MonoRight => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_routing_conversions() {
        assert_eq!(ChannelRouting::from_u8(0), ChannelRouting::Stereo);
        assert_eq!(ChannelRouting::from_u8(1), ChannelRouting::MonoLeft);
        assert_eq!(ChannelRouting::from_u8(2), ChannelRouting::MonoRight);
        assert_eq!(ChannelRouting::from_u8(99), ChannelRouting::Stereo);

        assert_eq!(ChannelRouting::Stereo.to_u8(), 0);
        assert_eq!(ChannelRouting::MonoLeft.to_u8(), 1);
        assert_eq!(ChannelRouting::MonoRight.to_u8(), 2);
    }

    #[test]
    fn test_channel_routing_output_channels() {
        // Multi-channel interface (e.g. 2, 4, 8 channels)
        assert_eq!(ChannelRouting::Stereo.output_channels(2), 2);
        assert_eq!(ChannelRouting::Stereo.output_channels(8), 2);
        assert_eq!(ChannelRouting::MonoLeft.output_channels(2), 1);
        assert_eq!(ChannelRouting::MonoRight.output_channels(2), 1);

        // Single-channel input device (mono mic)
        assert_eq!(ChannelRouting::Stereo.output_channels(1), 1);
        assert_eq!(ChannelRouting::MonoLeft.output_channels(1), 1);
        assert_eq!(ChannelRouting::MonoRight.output_channels(1), 1);
    }
}
