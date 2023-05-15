use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};

struct Zstd {
    level: i32,
}

impl DescribeScheme for Zstd {
    fn name(&self) -> String {
        "zstd".to_string()
    }
    fn settings(&self) -> Option<String> {
        Some(format!("level {}", self.level))
    }
}

impl Compressor for Zstd {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        zstd::bulk::compress(data, self.level).context("zstd compression failed")
    }
}

impl Decompressor for Zstd {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        let len =
            zstd::bulk::decompress_to_buffer(src, dst).context("zstd decompression failed")?;
        anyhow::ensure!(len == dst.len(), "dst buffer length mismatch");
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let mut schemes = vec![];
    for i in [-50, -20, -15, -10, -5, -2, -1] {
        schemes.push(Zstd { level: i });
    }
    for i in 0..=22 {
        schemes.push(Zstd { level: i });
    }
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
