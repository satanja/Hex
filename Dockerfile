FROM ubuntu:16.04

RUN apt-get update

RUN apt-get install -y \ 
    build-essential \ 
    curl \ 
    coinor-libcbc-dev

RUN apt-get update 

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /~/dfvstritus
COPY benches/ benches/
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY src/ src/

RUN cargo build --release