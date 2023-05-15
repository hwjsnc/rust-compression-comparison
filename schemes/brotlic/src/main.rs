use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};
use std::io::{Read, Write};

struct Brotli {
    quality: u8,
    window: WindowSize,
    block: BlockSize,
}

#[derive(Debug, Clone, Copy)]
enum WindowSize {
    Best,
    Worst,
}

#[derive(Debug, Clone, Copy)]
enum BlockSize {
    Best,
    Worst,
}

impl Into<brotlic::WindowSize> for WindowSize {
    fn into(self) -> brotlic::WindowSize {
        match self {
            WindowSize::Best => brotlic::WindowSize::best(),
            WindowSize::Worst => brotlic::WindowSize::worst(),
        }
    }
}

impl Into<brotlic::BlockSize> for BlockSize {
    fn into(self) -> brotlic::BlockSize {
        match self {
            BlockSize::Best => brotlic::BlockSize::best(),
            BlockSize::Worst => brotlic::BlockSize::worst(),
        }
    }
}

impl DescribeScheme for Brotli {
    fn name(&self) -> String {
        "brotlic".to_string()
    }
    fn settings(&self) -> Option<String> {
        Some(format!(
            "quality {} / {:?} window size / {:?} block size",
            self.quality, self.window, self.block
        ))
    }
}

impl Compressor for Brotli {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        let mut compressor = brotlic::CompressorWriter::with_encoder(
            brotlic::BrotliEncoderOptions::new()
                .quality(brotlic::Quality::new(self.quality).unwrap())
                .window_size(self.window.into())
                .block_size(self.block.into())
                .build()
                .context("couldn't create brotli encoder")?,
            vec![],
        );
        compressor
            .write_all(data)
            .context("brotli compression failed")?;
        compressor.into_inner().context("brotli compression failed")
    }
}

impl Decompressor for Brotli {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        let mut decompressor = brotlic::DecompressorReader::new(src);
        decompressor
            .read_exact(dst)
            .context("brotli decompression error")?;
        let mut tmp = [0u8];
        match decompressor.read_exact(&mut tmp) {
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(()),
            _ => Err(anyhow::Error::msg(
                "brotli decompression failed: dst too short",
            )),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut schemes = vec![];
    for quality in 0..=11 {
        for window in [WindowSize::Worst, WindowSize::Best] {
            for block in [BlockSize::Worst, BlockSize::Best] {
                schemes.push(Brotli {
                    quality,
                    window,
                    block,
                });
            }
        }
    }
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
