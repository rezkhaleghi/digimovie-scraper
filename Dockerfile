FROM rust:latest

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

WORKDIR /usr/src/app/target/release

EXPOSE 6969

# Set the startup command to run the binary
CMD ["cargo","run","--release"]