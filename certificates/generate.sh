#!/bin/bash -ex
# Generates a self-signed certificate to use for webtransport
# run this from the root folder
OUT=./certificates
openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -keyout $OUT/key.pem -out $OUT/cert.pem -days 999 -nodes -subj "/CN=localhost"
openssl x509 -in $OUT/cert.pem -noout -text
