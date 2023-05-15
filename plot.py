import csv
import matplotlib as mpl
import matplotlib.pyplot as plt
import random
from math import log

plt.style.use("bmh")

with open("results.csv") as f:
    all_results = list(csv.DictReader(f))

# silesia is the hardest corpus to compress
# for each scheme, check what the highest achieved compression for this corpus is
compression_ratio_by_scheme = {}
for row in all_results:
    if row["corpus"] == "silesia":  # the hardest to compress
        compression_ratio_by_scheme[row["scheme"]] = max(
            compression_ratio_by_scheme.get(row["scheme"], 1),
            float(row["compression ratio"]),
        )
compression_ratio_by_scheme["uncompressed"] = 0

# for easier comparison and better visualization, we group the schemes by what format they implement
scheme_groups = [
    ["uncompressed"],
    ["flate2 (rust)", "flate2 (zlib-ng)", "yazi"],
    ["deflate", "zopfli", "zopfli-rs"],
    ["lzzzz", "lz4_flex"],
    ["rust-lzma", "lzma-rs"],
    ["tetsy_snappy", "xsnappy", "snap"],
    ["brotlic", "brotli"],
    ["zstd"],
    ["lzo1x-1"],
    ["lzss (dyn)", "lzss (static)"],
    ["bzip2"],
]

# sort members of each group by highest compression ratio and groups by highest compression ratio within the group
# this way
scheme_groups = sorted(
    [
        sorted(
            group, key=lambda scheme: compression_ratio_by_scheme.get(scheme, 1)
        )  # TODO: no get
        for group in scheme_groups
    ],
    key=lambda group: max(
        compression_ratio_by_scheme.get(scheme, 1) for scheme in group
    ),
)

for group in scheme_groups:
    print(group)

schemes = [scheme for group in scheme_groups for scheme in group]

# check that all schemes are accounted for
for row in all_results:
    if row["scheme"] not in schemes:
        print(f'error: unexpected scheme "{scheme}"')
        exit()


# corpora = ["canterbury", "canterbury large", "silesia"]
corpora = list(reversed(["canterbury", "canterbury large", "silesia"]))

# base colors from colorbrewer: https://colorbrewer2.org/#type=qualitative&scheme=Paired&n=10
colors = {
    "uncompressed": "#000000",
    **{x: "#1f78b4" for x in ["flate2 (rust)", "flate2 (zlib-ng)", "yazi"]},
    **{x: "#33a02c" for x in ["deflate", "zopfli", "zopfli-rs"]},
    **{x: "#e31a1c" for x in ["lzzzz", "lz4_flex"]},
    **{x: "#a6cee3" for x in ["rust-lzma", "lzma-rs"]},
    **{x: "#b2df8a" for x in ["tetsy_snappy", "xsnappy", "snap"]},
    **{x: "#fdbf6f" for x in ["brotlic", "brotli"]},
    **{x: "#fb9a99" for x in ["zstd"]},
    **{x: "#cab2d6" for x in ["lzo1x-1"]},
    **{x: "#ff7f00" for x in ["lzss (dyn)", "lzss (static)"]},
    **{x: "#6a3d9a" for x in ["bzip2"]},
}
"""
colors = {
    "uncompressed": "#000000",
    **{x: "#1f78b4" for x in ["flate2 (rust)", "flate2 (zlib-ng)", "yazi"]},
    **{x: "#33a02c" for x in ["deflate", "zopfli", "zopfli-rs"]},
    "lzzzz": "#e31a1c",
    "lz4_flex": "#b00808",
    "rust-lzma": "#a6cee3",
    "lzma-rs": "#c2d6ff",
    **{x: "#b2df8a" for x in ["tetsy_snappy", "xsnappy", "snap"]},
    "lzss (dyn)": "#ff7f00",
    "lzss (static)": "#ffaf33",
    "brotli": "#d09749",
    "brotlic": "#fdbf6f",
    **{x: "#fb9a99" for x in ["zstd"]},
    **{x: "#cab2d6" for x in ["lzo1x-1"]},
    **{x: "#6a3d9a" for x in ["bzip2"]},
}
"""
markers = {
    "uncompressed": "o",
    "flate2 (rust)": "s",
    "flate2 (zlib-ng)": "d",
    "yazi": "o",
    "deflate": "o",
    "zopfli": "d",
    "zopfli-rs": "s",
    "lzzzz": "d",
    "lz4_flex": "o",
    "rust-lzma": "o",
    "lzma-rs": "s",
    "tetsy_snappy": "o",
    "xsnappy": "s",
    "snap": "d",
    "brotlic": "o",
    "brotli": "s",
    "zstd": "o",
    "lzo1x-1": "o",
    "lzss (dyn)": "o",
    "lzss (static)": "s",
    "bzip2": "o",
}


mpl.rcParams.update({"font.size": 14})
figsize = (30 / 2.54, 20 / 2.54)


def plot_compression_by_schemes(corpus):
    results = [row for row in all_results if row["corpus"] == corpus]
    plt.clf()
    plt.close()
    fig, ax = plt.subplots(figsize=figsize)

    ax.set_yticks([i / 10 for i in range(0, 11, 1)])
    ax.yaxis.set_minor_locator(mpl.ticker.MultipleLocator(0.025))

    plt.grid(visible=True, which="major", axis="both", linestyle="-")
    plt.grid(visible=True, which="minor", axis="both", linestyle=":")
    ax.set_axisbelow(True)

    for i, scheme in enumerate(schemes, 1):
        ax.scatter(
            [i for row in results if row["scheme"] == scheme],
            [
                1 - 1 / float(row["compression ratio"])
                for row in results
                if row["scheme"] == scheme
            ],
            label=scheme,
            color=colors[scheme],
            marker=markers[scheme],
            s=40,
        )

    # ax.legend(loc="upper left", fontsize="small", ncols=3)

    ax.text(
        0.925,
        0.975,
        f"corpus: {corpus}",
        size="large",
        horizontalalignment="right",
        verticalalignment="top",
        transform=ax.transAxes,
    )

    ax.set_xlabel("scheme")
    ax.set_ylabel("1 - compressed size / decompressed size\n(higher is better)")

    ax.set_ylim(-0.025, 1.025)

    plt.xticks(rotation=30, horizontalalignment="right")
    ax.set_xticks([i + 1 for i in range(0, len(schemes))])
    ax.set_xticklabels(schemes)

    plt.tight_layout()
    plt.savefig(f"plots/compression-{corpus}.png", dpi=300)


def plot_throughput_by_scheme(corpus):
    results = [row for row in all_results if row["corpus"] == corpus]

    # plot compression and decompression speed for all schemes
    plt.clf()
    plt.close()
    fig, ax = plt.subplots(figsize=figsize)

    ax.set_yticks([i / 10 for i in range(0, 11, 1)])
    ax.set_yscale("log")

    plt.grid(visible=True, which="major", axis="both", linestyle="-")
    plt.grid(visible=True, which="minor", axis="both", linestyle=":")
    ax.set_axisbelow(True)

    for i, scheme in enumerate(schemes, 1):
        ax.errorbar(
            [i - 0.1 for row in results if row["scheme"] == scheme],
            [
                float(row["compression speed (MB/s)"])
                for row in results
                if row["scheme"] == scheme
            ],
            yerr=[
                float(row["compression speed standard deviation (MB/s)"])
                for row in results
                if row["scheme"] == scheme
            ],
            # label = scheme,
            color=colors[scheme],
            marker="v",
            linestyle="none",
            # markersize = 40,
        )
        ax.errorbar(
            [
                i + 0.1
                for row in results
                if row["scheme"] == scheme and row["decompression speed (MB/s)"]
            ],
            [
                float(row["decompression speed (MB/s)"])
                for row in results
                if row["scheme"] == scheme and row["decompression speed (MB/s)"]
            ],
            yerr=[
                float(row["compression speed standard deviation (MB/s)"])
                for row in results
                if row["scheme"] == scheme and row["decompression speed (MB/s)"]
            ],
            # label = scheme,
            color=colors[scheme],
            marker="^",
            linestyle="none",
            # markersize = 40,
        )

    # TODO: legend with black v and ^ (compression/decompression)
    ax.legend(
        handles=[
            mpl.lines.Line2D([], [], color="black", label="compression", marker="v"),
            mpl.lines.Line2D([], [], color="black", label="decompression", marker="^"),
        ],
        loc="upper right",
        fontsize="small",
        ncols=1,
    )

    ax.text(
        0.025,
        0.025,
        f"corpus: {corpus}",
        size="large",
        horizontalalignment="left",
        verticalalignment="bottom",
        transform=ax.transAxes,
    )

    ax.set_xlabel("scheme")
    ax.set_ylabel("throughput (higher is better)")

    plt.xticks(rotation=30, horizontalalignment="right")
    ax.set_xticks([i + 1 for i in range(0, len(schemes))])
    ax.set_xticklabels(schemes)

    ax.set_ylim(0.1, 10_000)

    ax.set_yticks([0.1, 1, 10, 100, 1000, 10_000])
    ax.set_yticklabels(
        [
            *[f"{n} KB/s" for n in [100]],
            *[f"{n} MB/s" for n in [1, 10, 100]],
            *[f"{n} GB/s" for n in [1, 10]],
        ]
    )

    plt.tight_layout()
    plt.savefig(f"plots/throughput-{corpus}.png", dpi=300)


def plot_throughput_by_compression(corpus, which):
    results = [row for row in all_results if row["corpus"] == corpus]

    # plot compression speed versus compression quality
    plt.clf()
    plt.close()
    fig, ax = plt.subplots(figsize=figsize)

    ax.set_yticks([i / 10 for i in range(0, 11, 1)])
    ax.set_yscale("log")

    plt.grid(visible=True, which="major", axis="both", linestyle="-")
    plt.grid(visible=True, which="minor", axis="both", linestyle=":")
    ax.set_axisbelow(True)

    for i, scheme in enumerate(schemes, 1):
        ax.errorbar(
            [
                1 - 1 / float(row["compression ratio"])
                for row in results
                if row["scheme"] == scheme and row[f"{which} speed (MB/s)"]
            ],
            [
                float(row[f"{which} speed (MB/s)"])
                for row in results
                if row["scheme"] == scheme and row[f"{which} speed (MB/s)"]
            ],
            yerr=[
                float(row[f"{which} speed standard deviation (MB/s)"])
                for row in results
                if row["scheme"] == scheme and row[f"{which} speed (MB/s)"]
            ],
            label=scheme,
            color=colors[scheme],
            marker=markers[scheme],
            # marker="v" if which == "compression" else "^",
            linestyle="none",
            # markersize = 40,
        )

    ax.legend(loc="lower left", fontsize="small", ncols=3)

    ax.text(
        0.975,
        0.975,
        f"corpus: {corpus}",
        size="large",
        horizontalalignment="right",
        verticalalignment="top",
        transform=ax.transAxes,
    )

    ax.set_xlabel("1 - compressed size / uncompressed size (higher is better)")
    ax.set_ylabel(f"{which} speed (higher is better)")

    ax.set_xticks([float(i) / 10 for i in range(0, 11)])

    ax.set_xlim(-0.025, 1.025)
    ax.set_ylim(0.1, 10_000)

    ax.set_yticks([0.1, 1, 10, 100, 1000, 10_000])
    ax.set_yticklabels(
        [
            *[f"{n} KB/s" for n in [100]],
            *[f"{n} MB/s" for n in [1, 10, 100]],
            *[f"{n} GB/s" for n in [1, 10]],
        ]
    )

    plt.tight_layout()
    plt.savefig(f"plots/{which[0]}-dc-{corpus}.png", dpi=300)


def plot_decompression_speed_versus_compression_speed(corpus):
    results = [row for row in all_results if row["corpus"] == corpus]

    # plot compression speed versus compression quality
    plt.clf()
    plt.close()
    fig, ax = plt.subplots(figsize=(figsize[0], figsize[0]))

    ax.set_xscale("log")
    ax.set_yscale("log")

    plt.grid(visible=True, which="major", axis="both", linestyle="-")
    plt.grid(visible=True, which="minor", axis="both", linestyle=":")
    ax.set_axisbelow(True)

    for i, scheme in enumerate(schemes, 1):
        ax.errorbar(
            [
                float(row["compression speed (MB/s)"])
                for row in results
                if row["scheme"] == scheme and row["decompression speed (MB/s)"]
            ],
            [
                float(row["decompression speed (MB/s)"])
                for row in results
                if row["scheme"] == scheme and row["decompression speed (MB/s)"]
            ],
            xerr=[
                float(row["compression speed standard deviation (MB/s)"])
                for row in results
                if row["scheme"] == scheme and row["decompression speed (MB/s)"]
            ],
            yerr=[
                float(row["decompression speed standard deviation (MB/s)"])
                for row in results
                if row["scheme"] == scheme and row["decompression speed (MB/s)"]
            ],
            label=scheme,
            color=colors[scheme],
            marker=markers[scheme],
            linestyle="none",
            # markersize = 40,
        )

    ax.legend(loc="lower right", fontsize="small", ncols=3)

    ax.text(
        0.025,
        0.025,
        f"corpus: {corpus}",
        size="large",
        horizontalalignment="left",
        verticalalignment="bottom",
        transform=ax.transAxes,
    )

    ax.set_xlabel("compression speed (higher is better)")
    ax.set_ylabel("decompression speed (higher is better)")

    ax.set_ylim(0.1, 10_000)
    ax.set_xlim(0.1, 10_000)

    ax.set_xticks([0.1, 1, 10, 100, 1000, 10_000])
    ax.set_xticklabels(
        [
            *[f"{n} KB/s" for n in [100]],
            *[f"{n} MB/s" for n in [1, 10, 100]],
            *[f"{n} GB/s" for n in [1, 10]],
        ]
    )
    ax.set_yticks([0.1, 1, 10, 100, 1000, 10_000])
    ax.set_yticklabels(
        [
            *[f"{n} KB/s" for n in [100]],
            *[f"{n} MB/s" for n in [1, 10, 100]],
            *[f"{n} GB/s" for n in [1, 10]],
        ]
    )

    plt.tight_layout()
    plt.savefig(f"plots/cs-ds-{corpus}.png", dpi=300)


for corpus in corpora:
    plot_compression_by_schemes(corpus=corpus)
    plot_throughput_by_scheme(corpus=corpus)
    plot_throughput_by_compression(corpus=corpus, which="compression")
    plot_throughput_by_compression(corpus=corpus, which="decompression")
    plot_decompression_speed_versus_compression_speed(corpus=corpus)
