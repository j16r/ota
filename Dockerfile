FROM rustlang/rust:nightly AS build
WORKDIR /usr/src

# Create a dummy project and build the app's dependencies.
# If the Cargo.toml or Cargo.lock files have not changed,
# we can use the docker build cache and skip these (typically slow) steps.
RUN USER=root cargo new ota
WORKDIR /usr/src/ota
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# Copy the source and build the application.
COPY src ./src
RUN cargo install --path .

# Copy the statically-linked binary into a final minimal container
# FROM scratch
FROM debian:buster-slim
#RUN apt-get update && apt-get install -y extra-runtime-dependencies

WORKDIR /opt
COPY --from=build /usr/local/cargo/bin/ota .
COPY templates ./templates
COPY site ./site

USER 1000
CMD ["./ota"]
