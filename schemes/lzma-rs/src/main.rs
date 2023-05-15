use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};

#[derive(Debug)]
enum Lzma {
    Lzma,
    Lzma2,
    Xz,
}

impl DescribeScheme for Lzma {
    fn name(&self) -> String {
        "lzma-rs".to_string()
    }
    fn settings(&self) -> Option<String> {
        Some(format!("{:?}", self))
    }
}

impl Compressor for Lzma {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        let mut reader = std::io::Cursor::new(data);
        let mut vec = vec![];
        match self {
            Lzma::Lzma => {
                lzma_rs::lzma_compress(&mut reader, &mut vec).context("lzma compression failed")?
            }
            Lzma::Lzma2 => {
                lzma_rs::lzma2_compress(&mut reader, &mut vec).context("lzma compression failed")?
            }
            Lzma::Xz => {
                lzma_rs::xz_compress(&mut reader, &mut vec).context("lzma compression failed")?
            }
        }
        Ok(vec)
    }
}

impl Decompressor for Lzma {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        let expected_len = dst.len();
        let mut reader = std::io::Cursor::new(src);
        let mut writer = std::io::Cursor::new(dst);
        match self {
            Lzma::Lzma => lzma_rs::lzma_decompress(&mut reader, &mut writer)
                .context("lzma compression failed")?,
            Lzma::Lzma2 => lzma_rs::lzma2_decompress(&mut reader, &mut writer)
                .context("lzma compression failed")?,
            Lzma::Xz => lzma_rs::xz_decompress(&mut reader, &mut writer)
                .context("lzma compression failed")?,
        }
        let actual_len: usize = writer.position().try_into().unwrap();
        anyhow::ensure!(
            actual_len == expected_len,
            "lzma decompression error: dst oo short"
        );
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let schemes = [Lzma::Lzma, Lzma::Lzma2, Lzma::Xz];
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
