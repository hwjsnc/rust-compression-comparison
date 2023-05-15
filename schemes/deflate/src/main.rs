use anyhow::Context as _;
use common::{benchmark_compression_only, Compressor, DescribeScheme};
use std::io::Write as _;

struct Deflate {
    mode: deflate::Compression,
}

impl DescribeScheme for Deflate {
    fn name(&self) -> String {
        "deflate".to_string()
    }
    fn settings(&self) -> Option<String> {
        Some(format!("{:?}", self.mode))
    }
}

impl Compressor for Deflate {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        let mut encoder = deflate::write::DeflateEncoder::new(vec![], self.mode);
        encoder
            .write_all(data)
            .context("deflate compression failed")?;
        encoder.finish().context("deflate compression failed")
    }
}

fn main() -> anyhow::Result<()> {
    let schemes = [
        Deflate {
            mode: deflate::Compression::Fast,
        },
        Deflate {
            mode: deflate::Compression::Default,
        },
        Deflate {
            mode: deflate::Compression::Best,
        },
    ];
    benchmark_compression_only(std::io::stdout(), schemes).context("benchmark failed")
}
