## Execution, this builds a docker image to use as base for execution in amazonlinux with openssl
FROM amazonlinux:2023
RUN yum update -y
RUN yum install openssl -y