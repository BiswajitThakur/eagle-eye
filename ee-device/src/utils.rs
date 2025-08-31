use std::io::{self, BufReader, BufWriter, Read, Write};

use aes::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use ee_stream::EStreamSync;

pub(crate) fn handle_auth_on_sender_sync<'a>(
    key: [u8; 32],
    reader: impl io::Read + 'a,
    writer: impl io::Write + 'a,
) -> io::Result<Option<EStreamSync<'a>>> {
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);
    let mut iv = [0; 16];
    let mut buf = [0u8; 32];
    reader.read_exact(&mut iv)?;
    reader.read_exact(&mut buf)?;
    let mut cipher = ctr::Ctr64LE::<aes::Aes256>::new(&key.into(), &iv.into());
    cipher.apply_keystream(&mut buf);
    writer.write_all(&buf)?;
    writer.flush()?;
    reader.read_exact(&mut buf[0..3])?;
    if buf[0..3] != *b":0:" {
        return Ok(None);
    }
    cipher.seek(0);
    let e_stream = EStreamSync::builder()
        .cipher(cipher)
        .reader(reader)
        .writer(writer)
        .build();
    Ok(e_stream)
}
