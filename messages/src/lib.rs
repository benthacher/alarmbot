#![no_std]
use cobs;
use zerocopy::{CastError, FromBytes, Immutable, IntoBytes, KnownLayout};

pub const FRAME_DELIMITER: u8 = 0x00;

#[derive(Debug)]
pub enum MessageDecodeError {
    EmptyFrame,
    InvalidFrame,
    TargetBufTooSmall,
    AlignmentError,
    SizeError,
    UnknownMessageId,
}

#[derive(Debug)]
pub enum MessageEncodeError {
    TargetBufTooSmall,
}

impl From<cobs::DecodeError> for MessageDecodeError {
    fn from(value: cobs::DecodeError) -> Self {
        match value {
            cobs::DecodeError::EmptyFrame => MessageDecodeError::EmptyFrame,
            cobs::DecodeError::InvalidFrame { decoded_bytes: _ } => {
                MessageDecodeError::InvalidFrame
            }
            cobs::DecodeError::TargetBufTooSmall => MessageDecodeError::TargetBufTooSmall,
        }
    }
}
impl<Src, Dst> From<CastError<Src, Dst>> for MessageDecodeError {
    fn from(value: CastError<Src, Dst>) -> Self {
        match value {
            zerocopy::ConvertError::Alignment(_) => MessageDecodeError::AlignmentError,
            zerocopy::ConvertError::Size(_) => MessageDecodeError::SizeError,
        }
    }
}

pub trait EncodableMessage {
    fn encode(&self, buf: &mut [u8]) -> usize;
}

macro_rules! max {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = max!($($z),*);
        if $x > y {
            $x
        } else {
            y
        }
    }}
}

macro_rules! messages {
    (
        $($name:ident {
            $($field:ident: $type:ty),* $(,)?
            $(,)?
        } = $id:expr),*
        $(,)?
    ) => {
        $(#[derive(IntoBytes, FromBytes, KnownLayout, Immutable)]
        #[repr(C)]
        pub struct $name {
            pub $($field: $type),*
        })*

        pub enum Message<'a> {
            $($name(&'a $name)),*
        }

        pub const MAX_ENCODED_LENGTH: usize = 2 + max!(
            $(cobs::max_encoding_length(core::mem::size_of::<$name>()) + 1),*
        );

        impl<'a> TryFrom<&'a mut [u8]> for Message<'a> {
            type Error = MessageDecodeError;
            fn try_from(value: &'a mut [u8]) -> Result<Self, Self::Error> {
                cobs::decode_in_place_with_sentinel(value, FRAME_DELIMITER)?;

                match value[0] {
                    $($id => Ok(Self::$name($name::ref_from_bytes(&value[1..])?))),*,
                    _ => Err(Self::Error::UnknownMessageId)
                }
            }
        }

        $(impl EncodableMessage for $name {
            fn encode(&self, buf: &mut [u8]) -> usize {
                buf[0] = $id;
                let encodable_slice = &mut buf[1..];
                let written = cobs::encode_with_sentinel(self.as_bytes(), encodable_slice, FRAME_DELIMITER);
                encodable_slice[written] = FRAME_DELIMITER;

                written
            }
        })*
    };
}

messages!(
    HeightSetpoint { height: u16 } = 0,
    OtherThing { value: u8 } = 1,
);

pub fn init() {
    let height_cmd = HeightSetpoint { height: 3 };
    // cobs::decode_in_place(buff)

    // height_cmdconst .max_encoded_length()
}
