#!/bin/bash

# Experimental!
#
# Downloads, builds and installs the latest version.
# At the moment, this process requires Git and Cargo.

mkdir $HOME/.hs-install
cd $HOME/.hs-install

git clone https://github.com/dominikbraun/haystack haystack
cd haystack

cargo build --release
sudo cp target/release/haystack /usr/local/bin

rm -rf $HOME/.hs-install
