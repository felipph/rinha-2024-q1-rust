FROM rust as builder
WORKDIR /usr/src/rinha
COPY . .
RUN cargo install --path .
CMD ["rinha"]
# FROM debian:buster-slim
# RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
# COPY --from=builder /usr/local/cargo/bin/rinha /usr/local/bin/rinha
# CMD ["rinha"]