use packets::read::{ByteReader, ByteReadable};

pub struct PacketData {
    pub id: u32,
    pub reader: ByteReader
}

impl PacketData {

    #[inline]
    pub fn read<T: ByteReadable>(&mut self) -> T{
        self.reader.read()
    }
}
