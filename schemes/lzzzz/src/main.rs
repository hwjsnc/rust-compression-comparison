use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};
use lzzzz::lz4;
use lzzzz::lz4_hc;
use lzzzz::lz4f;
use std::io::Read as _;

enum Lz4 {
    NormalBlock(i32),
    HcBlock(i32),
    Frame(Level),
}

#[derive(Debug)]
enum Level {
    Default,
    High,
    Max,
}

impl DescribeScheme for Lz4 {
    fn name(&self) -> String {
        "lzzzz".to_string()
    }
    fn settings(&self) -> Option<String> {
        Some(match self {
            Lz4::NormalBlock(a) => format!("block / normal / acceleration {a}"),
            Lz4::HcBlock(a) => format!("block / hc / acceleration {a}"),
            Lz4::Frame(level) => format!("frame / {level:?}"),
        })
    }
}

impl Compressor for Lz4 {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        let mut vec = vec![];
        let len = match self {
            Lz4::NormalBlock(level) => {
                lz4::compress_to_vec(data, &mut vec, *level).context("lzzzz error")?
            }
            Lz4::HcBlock(level) => {
                lz4_hc::compress_to_vec(data, &mut vec, *level).context("lzzzz error")?
            }
            Lz4::Frame(level) => {
                let preferences = match level {
                    Level::Default => lz4f::PreferencesBuilder::default().build(),
                    Level::High => lz4f::PreferencesBuilder::new()
                        .compression_level(lz4f::CLEVEL_HIGH)
                        .build(),
                    Level::Max => lz4f::PreferencesBuilder::new()
                        .compression_level(lz4f::CLEVEL_MAX)
                        .build(),
                };
                lz4f::compress_to_vec(data, &mut vec, &preferences).context("lzzzz error")?
            }
        };
        anyhow::ensure!(len == vec.len());
        Ok(vec)
    }
}

impl Decompressor for Lz4 {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        match self {
            Lz4::NormalBlock(_) | Lz4::HcBlock(_) => {
                let len = lz4::decompress(src, dst).context("lzzzz error")?;
                anyhow::ensure!(len == dst.len(), "destination buffer length doesn't match");
            }
            Lz4::Frame(_) => {
                let mut decompressor = lz4f::ReadDecompressor::new(src)
                    .context("couldn't initialize the lzzzz decompressor")?;
                decompressor
                    .read_exact(dst)
                    .context("lzzzz decompression error")?;
                let remaining = decompressor.into_inner();
                anyhow::ensure!(remaining.is_empty(), "destination buffer too short");
            }
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let mut schemes = vec![];
    schemes.push(Lz4::Frame(Level::Default));
    schemes.push(Lz4::Frame(Level::High));
    schemes.push(Lz4::Frame(Level::Max));
    for i in [0, 1, 2, 4, 8, 16, 128, 1024, 8192, 65535] {
        schemes.push(Lz4::NormalBlock(i));
        schemes.push(Lz4::HcBlock(i));
    }
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
