FROM rust:1.49 as builder

WORKDIR /usr/src/typewriters
COPY . .

RUN cargo install --path .

FROM debian:buster-slim

RUN apt-get update \
    && apt-get install -y extra-runtime-dependencies \
	&& rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/typewriters /usr/local/bin/typewriters

ARG data=/data
ARG uid=1000
ARG gid=1000
ARG port=8080

VOLUME $data
WORKDIR $data

USER $uid:$gid

EXPOSE $port

CMD ["typewriters"]
