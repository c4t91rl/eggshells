Go into `cd server_data/certs`

openssl genpkey -algorithm RSA -out key.pem -pkeyopt rsa_keygen_bits:2048
openssl req -new -key key.pem -out csr.pem

Fill firts with `PL` the rest enter

openssl req -x509 \
        -key key.pem \
        -out cert.pem \
        -days 365 \
        -subj "/C=PL/ST=SomeState/O=MyOrg/CN=127.0.0.1" \
        -addext "subjectAltName=IP:127.0.0.1,DNS:localhost"
