FROM archlinux/archlinux:latest
FROM rust:latest
WORKDIR /usr/app
COPY src /usr/app
RUN cargo install --path .
CMD [ "/bin/bash" ]