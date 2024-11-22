# Generate a private key & self-signed certificate
gen-selfsigned-cert:
  openssl req -new -newkey rsa:2048 -nodes -keyout key.pem -subj "/CN=localhost\/emailAddress=root@localhost/" -out localhost.csr
  openssl x509 -signkey key.pem -in localhost.csr -req -days 10240 -out localhost.crt
