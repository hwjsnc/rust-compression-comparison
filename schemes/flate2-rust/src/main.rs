use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};
use std::io::{Read, Write};

enum Deflate {
    Deflate(flate2::Compression),
    Zlib(flate2::Compression),
    GZip(flate2::Compression),
}

impl DescribeScheme for Deflate {
    fn name(&self) -> String {
        "flate2 (rust)".to_string()
    }
    fn settings(&self) -> Option<String> {
        match self {
            Deflate::Deflate(c) => Some(format!("deflate / level {}", c.level())),
            Deflate::Zlib(c) => Some(format!("zlib / level {}", c.level())),
            Deflate::GZip(c) => Some(format!("gzip / level {}", c.level())),
        }
    }
}

impl Compressor for Deflate {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        match self {
            Deflate::Deflate(level) => {
                let mut encoder = flate2::write::DeflateEncoder::new(vec![], *level);
                encoder
                    .write_all(data)
                    .context("deflate compression failed")?;
                encoder.finish().context("deflate compression failed")
            }
            Deflate::Zlib(level) => {
                let mut encoder = flate2::write::ZlibEncoder::new(vec![], *level);
                encoder
                    .write_all(data)
                    .context("deflate compression failed")?;
                encoder.finish().context("zlib compression failed")
            }
            Deflate::GZip(level) => {
                let mut encoder = flate2::write::GzEncoder::new(vec![], *level);
                encoder
                    .write_all(data)
                    .context("deflate compression failed")?;
                encoder.finish().context("gzip compression failed")
            }
        }
    }
}

impl Decompressor for Deflate {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        match self {
            Deflate::Deflate(_) => {
                let mut decoder = flate2::read::DeflateDecoder::new(src);
                decoder
                    .read_exact(dst)
                    .context("deflate decompression failed")?;
                let mut tmp = [0u8];
                match decoder.read_exact(&mut tmp) {
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(()),
                    _ => Err(anyhow::Error::msg(
                        "deflate decompression failed: dst too short",
                    )),
                }
            }
            Deflate::Zlib(_) => {
                let mut decoder = flate2::read::ZlibDecoder::new(src);
                decoder
                    .read_exact(dst)
                    .context("deflate decompression failed")?;
                let mut tmp = [0u8];
                match decoder.read_exact(&mut tmp) {
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(()),
                    _ => Err(anyhow::Error::msg(
                        "deflate decompression failed: dst too short",
                    )),
                }
            }
            Deflate::GZip(_) => {
                let mut decoder = flate2::read::GzDecoder::new(src);
                decoder
                    .read_exact(dst)
                    .context("deflate decompression failed")?;
                let mut tmp = [0u8];
                match decoder.read_exact(&mut tmp) {
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(()),
                    _ => Err(anyhow::Error::msg(
                        "deflate decompression failed: dst too short",
                    )),
                }
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut schemes = vec![];
    for level in 0..=10 {
        schemes.push(Deflate::Deflate(flate2::Compression::new(level)));
        schemes.push(Deflate::Zlib(flate2::Compression::new(level)));
        schemes.push(Deflate::GZip(flate2::Compression::new(level)));
    }
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
