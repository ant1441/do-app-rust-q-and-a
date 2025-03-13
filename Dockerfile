# -- Stage 1 -- #
# Compile the app.
FROM rust:slim as builder

RUN USER=root cargo new --bin q-and-a

WORKDIR /q-and-a

COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/q_and_a*

RUN cargo build --release


# -- Stage 2 -- #
# Create the final environment with the compiled binary.
FROM debian:bookworm-slim as runtime

WORKDIR /bin

# Copy from builder and rename to 'server'
COPY --from=builder /q-and-a/target/release/q-and-a ./server

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC \
    USER=appuser \
    ROCKET_ADDRESS=:: \
    ROCKET_PORT=8080

RUN groupadd ${USER} \
    && useradd -g ${USER} ${USER} && \
    chown -R ${USER}:${USER} /bin

USER ${USER}

ENTRYPOINT ["./server"]
