# -- Stage 1 -- #
# Compile the app.
FROM rust:slim as builder

RUN USER=root cargo new --bin q-and-a
RUN apt-get update \
    && apt-get install -y libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /q-and-a

# Ensure SQLX uses the saved query metadata in .sqlx
ENV SQLX_OFFLINE true
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
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
COPY static .

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
