
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y libssl-dev
RUN apt-get install -y ca-certificates
WORKDIR /app
COPY target/release/ledger-cli .
CMD ["./ledger-cli"]