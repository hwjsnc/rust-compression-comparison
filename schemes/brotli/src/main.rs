use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};
use std::io::Write;

struct Brotli {
    quality: u32,
    window_size: u32,
    buffer_size: usize,
}

impl DescribeScheme for Brotli {
    fn name(&self) -> String {
        "brotli".to_string()
    }
    fn settings(&self) -> Option<String> {
        Some(format!(
            "quality {} / window size {} / buffer size {}",
            self.quality, self.window_size, self.buffer_size
        ))
    }
}

impl Compressor for Brotli {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        let mut compressor =
            brotli::CompressorWriter::new(vec![], self.buffer_size, self.quality, self.window_size);
        compressor
            .write_all(data)
            .context("brotli compression failed")?;
        Ok(compressor.into_inner())
    }
}

impl Decompressor for Brotli {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        let expected_len = dst.len();
        let mut cursor = std::io::Cursor::new(dst);
        brotli::BrotliDecompress(&mut std::io::Cursor::new(src), &mut cursor)
            .context("brotli decompression failed")?;
        let actual_len: usize = cursor.position().try_into().unwrap();
        assert_eq!(actual_len, expected_len);
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let mut schemes = vec![];
    for quality in 0..=11 {
        for window_size in [20, 21, 22] {
            for buffer_size in [4096] {
                schemes.push(Brotli {
                    quality,
                    window_size,
                    buffer_size,
                });
            }
        }
    }
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
