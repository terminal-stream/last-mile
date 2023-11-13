## Setup an amazonlinux openssl rust build environment
FROM amazonlinux:2023 as builder
RUN yum update -y
RUN yum groupinstall "Development Tools" -y
RUN yum install pkg-config gcc -y
RUN yum install perl openssl-devel openssl-libs openssl -y
RUN yum install protobuf protobuf-compiler -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y