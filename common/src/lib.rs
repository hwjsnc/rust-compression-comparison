use anyhow::Context as _;
use average::Estimate as _;
use std::borrow::BorrowMut as _;

pub const SAMPLES: std::num::NonZeroU64 = match std::num::NonZeroU64::new(10) {
    Some(v) => v,
    None => [][0],
}; // see https://stackoverflow.com/a/66838483 for the source of this monstrosity

pub trait Compressor {
    fn compress(&self, data: &[u8]) -> anyhow::Result<std::vec::Vec<u8>>;
}

pub trait Decompressor {
    fn decompress_to(&self, src: &[u8], dst: &mut [u8]) -> anyhow::Result<()>;
}

pub trait DescribeScheme {
    fn name(&self) -> String;
    fn settings(&self) -> Option<String>;
}

pub struct Corpus {
    pub name: &'static str,
    pub data: std::vec::Vec<u8>,
}

pub fn read_corpus_data<P: AsRef<std::path::Path>, I: IntoIterator<Item = P>>(
    base: P,
    files: I,
) -> anyhow::Result<std::vec::Vec<u8>> {
    let mut result = vec![];
    for filename in files {
        let path = base.as_ref().join(filename);
        let mut reader = std::fs::File::open(&path)
            .with_context(|| format!("couldn't open file {}", path.display()))?;
        std::io::copy(&mut reader, &mut result)
            .with_context(|| format!("couldn't read from file {}", path.display()))?;
    }
    Ok(result)
}

pub fn read_corpora() -> anyhow::Result<std::vec::Vec<Corpus>> {
    let base = "../../corpora/";
    let canterbury = Corpus {
        name: "canterbury",
        data: read_corpus_data(
            base,
            [
                "canterbury/alice29.txt",
                "canterbury/asyoulik.txt",
                "canterbury/cp.html",
                "canterbury/fields.c",
                "canterbury/grammar.lsp",
                "canterbury/kennedy.xls",
                "canterbury/lcet10.txt",
                "canterbury/plrabn12.txt",
                "canterbury/ptt5",
                "canterbury/sum",
                "canterbury/xargs.1",
            ],
        )
        .context("couldn't read canterbury corpus")?,
    };
    anyhow::ensure!(
        canterbury.data.len() == 2_810_784,
        "canterbury corpus has unexpected size"
    );

    let canterbury_large = Corpus {
        name: "canterbury large",
        data: read_corpus_data(
            base,
            [
                "canterbury-large/E.coli",
                "canterbury-large/bible.txt",
                "canterbury-large/world192.txt",
            ],
        )
        .context("couldn't read canterbury large corpus")?,
    };
    anyhow::ensure!(
        canterbury_large.data.len() == 11_159_482,
        "canterbury large corpus has unexpected size"
    );

    let silesia = Corpus {
        name: "silesia",
        data: read_corpus_data(
            base,
            [
                "silesia/dickens",
                "silesia/mozilla",
                "silesia/mr",
                "silesia/nci",
                "silesia/ooffice",
                "silesia/osdb",
                "silesia/reymont",
                "silesia/samba",
                "silesia/sao",
                "silesia/webster",
                "silesia/xml",
                "silesia/x-ray",
            ],
        )
        .context("couldn't read silesia corpus")?,
    };
    anyhow::ensure!(
        silesia.data.len() == 211_938_580,
        "silesia corpus has unexpected size"
    );

    Ok(vec![canterbury, canterbury_large, silesia])
}

#[derive(serde::Serialize)]
pub struct Result {
    pub scheme: String,
    pub settings: Option<String>,
    pub corpus: &'static str,
    pub compression_speed: f64,
    pub compression_speed_std: f64,
    pub decompression_speed: Option<f64>,
    pub decompression_speed_std: Option<f64>,
    pub compression_ratio: f64,
}

pub fn print_result<W: std::io::Write, R: std::borrow::Borrow<Result>>(
    f: &mut W,
    result: R,
) -> anyhow::Result<()> {
    let mut writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Necessary)
        .has_headers(false)
        .from_writer(f.borrow_mut());
    writer.serialize(result.borrow())?;
    Ok(())
}

fn time<F, R>(f: F) -> anyhow::Result<(R, std::time::Duration)>
where
    F: FnOnce() -> anyhow::Result<R>,
{
    let time = std::time::Instant::now();
    let result = f()?;
    let duration = time.elapsed();
    Ok((result, duration))
}

fn benchmark_scheme<C: Compressor + Decompressor + DescribeScheme + ?Sized>(
    scheme: &C,
    corpus: &Corpus,
    samples: std::num::NonZeroU64,
) -> anyhow::Result<Result> {
    let mut compressed_size = None;
    let corpus_size_mb: f64 = corpus.data.len() as f64 / 1_000_000.0f64;
    let mut compression_speed_mbps = average::MeanWithError::new();
    let mut decompression_speed_mbps = average::MeanWithError::new();
    for _ in 0..samples.get() {
        // compress
        let (compressed, t) =
            time(|| scheme.compress(&corpus.data)).context("couldn't time compression")?;
        if let Some(size) = compressed_size {
            anyhow::ensure!(
                size == compressed.len(),
                "compressed data size changed during runs"
            );
        } else {
            compressed_size = Some(compressed.len());
        }
        compression_speed_mbps.add(corpus_size_mb / t.as_secs_f64());

        // decompress
        let mut decompressed = vec![0u8; corpus.data.len()];
        let ((), t) = time(|| scheme.decompress_to(&compressed, &mut decompressed[..]))
            .context("couldn't time decompression")?;
        anyhow::ensure!(
            decompressed == corpus.data,
            "CRITICAL BUG: decompress(compress(x)) != x"
        );
        decompression_speed_mbps.add(corpus_size_mb / t.as_secs_f64());
    }

    let compressed_size = compressed_size.expect("must be set because sample size is nonzero");

    Ok(Result {
        scheme: scheme.name(),
        settings: scheme.settings(),
        corpus: corpus.name,
        compression_speed: compression_speed_mbps.mean(),
        compression_speed_std: compression_speed_mbps.sample_variance().sqrt(),
        decompression_speed: Some(decompression_speed_mbps.mean()),
        decompression_speed_std: Some(decompression_speed_mbps.sample_variance().sqrt()),
        compression_ratio: (corpus.data.len() as f64) / (compressed_size as f64),
    })
}

pub fn benchmark<
    W: std::io::Write,
    F: std::borrow::BorrowMut<W>,
    C: Compressor + Decompressor + DescribeScheme + ?Sized,
    S: std::borrow::Borrow<C>,
    I: IntoIterator<Item = S>,
>(
    mut f: F,
    schemes: I,
) -> anyhow::Result<()> {
    let corpora = read_corpora().context("couldn't read corpora")?;
    for scheme in schemes {
        for corpus in corpora.iter() {
            let result =
                benchmark_scheme(scheme.borrow(), &corpus, SAMPLES).with_context(|| {
                    if let Some(settings) = scheme.borrow().settings() {
                        format!(
                            "benchmark failed for scheme {} (settings '{}') with corpus {}",
                            scheme.borrow().name(),
                            settings,
                            corpus.name
                        )
                    } else {
                        format!(
                            "benchmark failed for scheme {} with corpus {}",
                            scheme.borrow().name(),
                            corpus.name
                        )
                    }
                })?;
            print_result(f.borrow_mut(), result).context("couldn't print result to stdout")?;
        }
    }
    Ok(())
}

fn benchmark_compression_scheme<C: Compressor + DescribeScheme>(
    scheme: &C,
    corpus: &Corpus,
    samples: std::num::NonZeroU64,
) -> anyhow::Result<Result> {
    let mut compressed_size = None;
    let corpus_size_mb: f64 = corpus.data.len() as f64 / 1_000_000.0f64;
    let mut compression_speed_mbps = average::MeanWithError::new();
    for _ in 0..samples.get() {
        // compress
        let (compressed, t) =
            time(|| scheme.compress(&corpus.data)).context("couldn't time compression")?;
        if let Some(size) = compressed_size {
            anyhow::ensure!(
                size == compressed.len(),
                "compressed data size changed during runs"
            );
        } else {
            compressed_size = Some(compressed.len());
        }
        compression_speed_mbps.add(corpus_size_mb / t.as_secs_f64());
    }

    let compressed_size = compressed_size.expect("must be set because sample size is nonzero");

    Ok(Result {
        scheme: scheme.name(),
        settings: scheme.settings(),
        corpus: corpus.name,
        compression_speed: compression_speed_mbps.mean(),
        compression_speed_std: compression_speed_mbps.sample_variance().sqrt(),
        decompression_speed: None,
        decompression_speed_std: None,
        compression_ratio: (corpus.data.len() as f64) / (compressed_size as f64),
    })
}

pub fn benchmark_compression_only<
    W: std::io::Write,
    F: std::borrow::BorrowMut<W>,
    C: Compressor + DescribeScheme,
    S: std::borrow::Borrow<C>,
    I: IntoIterator<Item = S>,
>(
    mut f: F,
    schemes: I,
) -> anyhow::Result<()> {
    let corpora = read_corpora().context("couldn't read corpora")?;
    for scheme in schemes {
        for corpus in corpora.iter() {
            let result = benchmark_compression_scheme(scheme.borrow(), &corpus, SAMPLES)
                .with_context(|| {
                    if let Some(settings) = scheme.borrow().settings() {
                        format!(
                            "benchmark failed for scheme {} (settings '{}') with corpus {}",
                            scheme.borrow().name(),
                            settings,
                            corpus.name
                        )
                    } else {
                        format!(
                            "benchmark failed for scheme {} with corpus {}",
                            scheme.borrow().name(),
                            corpus.name
                        )
                    }
                })?;
            print_result(f.borrow_mut(), result).context("couldn't print result to stdout")?;
        }
    }
    Ok(())
}
