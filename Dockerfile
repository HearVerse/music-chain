# Use the Parity CI Linux image as a base
FROM paritytech/ci-linux:production as builder
WORKDIR /substrate

# Copy the entire project folder into the Docker container
COPY . /substrate

# Start a new build stage and use the same base image
FROM paritytech/ci-linux:production

# Copy the entire project folder from the builder stage to the new stage
COPY --from=builder /substrate /substrate

# Set the working directory
WORKDIR /substrate

# Build the project and run the compiled binary
ENTRYPOINT cargo build --release && ./target/release/substrate --dev
