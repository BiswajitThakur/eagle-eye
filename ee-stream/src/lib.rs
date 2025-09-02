pub mod buffer;
pub mod e_stream;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowControl {
    Close = 0,
    Continue = 1,
    StopServer = 2,
}

impl TryFrom<[u8; 1]> for FlowControl {
    type Error = ();
    fn try_from(value: [u8; 1]) -> Result<Self, Self::Error> {
        match value {
            [0] => Ok(Self::Close),
            [1] => Ok(Self::Continue),
            [2] => Ok(Self::StopServer),
            _ => Err(()),
        }
    }
}

impl FlowControl {
    #[inline]
    pub fn to_be_bytes(&self) -> [u8; 1] {
        let v = *self as u8;
        v.to_be_bytes()
    }
}
