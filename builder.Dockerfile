FROM rust:latest as builder

RUN apt-get -qq update -y
RUN apt-get -qq install -y git wget clang cmake



# Inject some package dependency, If not do steps below, Build will pull dependency and compile it every time,  It cost to much time
# If code or cargo.toml has big chagne, You can rebuild builder image
ADD . /RETREE
WORKDIR /RETREE

# Add china crate registry because of china great firewall
RUN echo '[source.crates-io]\nregistry = "https://github.com/rust-lang/crates.io-index"\nreplace-with = "ustc"\n[source.ustc]\nregistry = "git://mirrors.ustc.edu.cn/crates.io-index"' >> /usr/local/cargo/config

# Build the source
RUN set -ex ;\
    cargo build --release ;\