FROM gitpod/workspace-full:latest
USER root
RUN apt update
RUN apt install libgtk-3-dev -y
RUN apt install gcc -y
USER gitpod
ENV PATH="$HOME/.cargo/bin:$PATH"
RUN rustup install 1.37.0
RUN rustup default 1.37.0
RUN rustup component add rls rust-analysis clippy rust-src