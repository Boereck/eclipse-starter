FROM rust:latest
RUN apt update
RUN apt install libgtk-3-dev -y
RUN apt install gcc -y
RUN rustup component add rls rust-analysis clippy