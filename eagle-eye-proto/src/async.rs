use tokio::io::{AsyncRead, AsyncWrite, BufReader, BufWriter};

pub struct EagleEyeStreamAsync<const N: usize, R: AsyncRead, W: AsyncWrite> {
    read_cipher: ctr::Ctr64LE<aes::Aes256>,
    write_cipher: ctr::Ctr64LE<aes::Aes256>,
    write_buff: [u8; N],
    reader: BufReader<R>,
    writer: BufWriter<W>,
}

impl<const N: usize, R: AsyncRead, W: AsyncWrite> AsyncRead for EagleEyeStreamAsync<N, R, W> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        todo!()
    }
}
