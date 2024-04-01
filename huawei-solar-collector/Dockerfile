FROM rust:1.65.0
WORKDIR /usr/src/

# create dummy project to cache dependencies
RUN cargo new collector
COPY Cargo.toml Cargo.lock /usr/src/collector/

# build to download and save dependencies
WORKDIR /usr/src/collector
RUN cargo build --release

# copy real source build project and only keep executable
COPY src/main.rs ./src/main.rs
RUN find ./src -exec touch {} +
RUN cargo build --release

# set executable as default
CMD ["cargo", "run", "--release"]
