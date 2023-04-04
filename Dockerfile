FROM paritytech/ci-linux:production as build

WORKDIR /music-chain
COPY . .
RUN cargo build --release

FROM ubuntu:20.04
WORKDIR /node

# Copy the node binary.
COPY --from=build /music-chain/target/release/substrate .

# Install root certs, see: https://github.com/paritytech/substrate/issues/9984
RUN apt update && \
    apt install -y ca-certificates && \
    update-ca-certificates && \
    apt remove ca-certificates -y && \
    rm -rf /var/lib/apt/lists/*

EXPOSE 9944
# Exposing unsafe RPC methods is needed for testing but should not be done in
# production.
CMD [ "./substrate", "--dev", "--ws-external", "--rpc-methods=Unsafe" ]
