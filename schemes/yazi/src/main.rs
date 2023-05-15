use anyhow::Context as _;
use common::{benchmark, Compressor, Decompressor, DescribeScheme};

struct Yazi {
    level: yazi::CompressionLevel,
}

impl DescribeScheme for Yazi {
    fn name(&self) -> String {
        "yazi".to_string()
    }
    fn settings(&self) -> Option<String> {
        Some(format!("{:?}", self.level))
    }
}

impl Compressor for Yazi {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>> {
        yazi::compress(data, yazi::Format::Raw, self.level)
            .map_err(|e| anyhow::Error::msg(format!("{e:?}")))
            .context("yazi compression failed")
    }
}

impl Decompressor for Yazi {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()> {
        let expected_len = dst.len();
        let mut decoder = yazi::Decoder::new();
        let mut stream = decoder.stream_into_buf(dst);
        stream
            .write(src)
            .map_err(|e| anyhow::Error::msg(format!("{e:?}")))
            .context("yazi decompression error")?;
        let (actual_len, _checksum) = stream
            .finish()
            .map_err(|e| anyhow::Error::msg(format!("{e:?}")))
            .context("yazi decompression error")?;
        let actual_len: usize = actual_len.try_into().unwrap();
        anyhow::ensure!(
            actual_len == expected_len,
            "yazi decompression error: compressed data too short"
        );
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let mut schemes = vec![
        Yazi {
            level: yazi::CompressionLevel::None,
        },
        Yazi {
            level: yazi::CompressionLevel::BestSpeed,
        },
        Yazi {
            level: yazi::CompressionLevel::Default,
        },
        Yazi {
            level: yazi::CompressionLevel::BestSize,
        },
    ];
    for level in 1..=10 {
        schemes.push(Yazi {
            level: yazi::CompressionLevel::Specific(level),
        });
    }
    benchmark(std::io::stdout(), schemes).context("benchmark failed")
}
