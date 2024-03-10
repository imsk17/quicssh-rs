FROM rust:1.76
EXPOSE 4242
WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .

CMD ["quicssh-rs", "server"]