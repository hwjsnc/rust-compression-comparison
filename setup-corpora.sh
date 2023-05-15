set -ex

# check presence of unzip programs
which tar
which unzip
which sha256sum

test cantrbry.tar.gz
test large.tar.gz
test silesia.zip
#curl 'https://sun.aei.polsl.pl/~sdeor/corpus/silesia.zip' --remote-name

sha256sum --check - > /dev/null <<EOF
f140e8a5b73d3f53198555a63bfb827889394a42f20825df33c810c3d5e3f8fb  cantrbry.tar.gz
7df00ff4cb8c9ce4187d9fa4100aa88208a72d9a227e3cd8bab23b71971eaf41  large.tar.gz
0626e25f45c0ffb5dc801f13b7c82a3b75743ba07e3a71835a41e3d9f63c77af  silesia.zip
EOF

mkdir -p corpora/{canterbury,canterbury-large,silesia}

tar xz -Ccorpora/canterbury -f cantrbry.tar.gz
tar xz -Ccorpora/canterbury-large -f large.tar.gz
unzip silesia.zip -d corpora/silesia

find corpora -type f -exec chmod -x {} ';'
