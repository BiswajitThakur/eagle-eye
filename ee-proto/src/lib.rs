pub trait EeProto: Sized + PartialEq + PartialOrd {
    fn major(&self) -> u16;
    fn minor(&self) -> u8;
    fn patch(&self) -> u8;
    fn version(&self) -> (u16, u8, u8) {
        (self.major(), self.minor(), self.patch())
    }
    fn to_bytes(&self) -> [u8; 4] {
        let major = self.major().to_be_bytes();
        let minor = self.minor().to_be_bytes();
        let patch = self.patch().to_be_bytes();
        [major[0], major[1], minor[0], patch[0]]
    }
}
