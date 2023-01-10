# Configuration of Hephaestus

Hephaestus is a single procedure and compiled into a single binary file. When hephaestus is started, configuration has to be provided via program paramater. Sample configuration file:
```
*
* Host addresses
*
host.grpc.address = localhost:9150             // gRPC host address
host.grpc.tls = yes                              // yes or no to enable/disable tls
host.grpc.tls.key = /etc/olympus/hephaestus/certs/hepha_pr_localhost.key
host.grpc.tls.pem = /etc/olympus/hephaestus/certs/hepha_pr_localhost.pem

*
* Plan related settings
*
plan.rule_dir = /etc/olympus/hephaestus/plans
plan.rule_log = /etc/olympus/hephaestus/logs

*
* Fill these to allow escalate statuses to Hermes
*
hermes.enable = yes
hermes.grpc.address = http://localhost:9099     // gRPC address and port of Hermes
hermes.grpc.tls = no
* hermes.grpc.tls.ca_cert = /placeholder
* hermes.grpc.tls.domain = placeholder
hermes.table = Hephaestus                         // Which table should the records send
```

Communication with Hephaestus can be done via gRPC calls. It can be secured (by TLS option) or non-secured too. If enabled, Hephaestus propagate every single plan final status to Hermes. From there it can be processed and response can be automated.

Hephaestus plans are those files which can contain more complext instruction which consist of steps.
Plans are stored in files, so it is easy to edit them and using XML-like syntax.

After a proper startup, output of program look like, when Hermes client also enabled:
```
hephaestus[3898172]: Version v.0.2.0 is starting...
hephaestus[3898172]: Configuration:
hephaestus[3898172]: host.grpc.tls -> yes
hephaestus[3898172]: plan.rule_dir -> /etc/olympus/hephaestus/plans
hephaestus[3898172]: hermes.grpc.address -> http://localhost:9099
hephaestus[3898172]: hermes.enable -> yes
hephaestus[3898172]: host.grpc.tls.key -> /etc/olympus/hephaestus/certs/hepha_pr_localhost.key
hephaestus[3898172]: hermes.grpc.tls -> no
hephaestus[3898172]: host.grpc.tls.pem -> /etc/olympus/hephaestus/certs/hepha_pr_localhost.pem
hephaestus[3898172]: plan.rule_log -> /etc/olympus/hephaestus/logs
hephaestus[3898172]: host.grpc.address -> localhost:9150
hephaestus[3898172]: hermes.table -> Hephaestus
hephaestus[3898172]: Directory check is OK
hephaestus[3898172]: Corresponse properties are set to yes, so start Hermes client
hephaestus[3898172]: Start gRPC endpoint in on 192.168.50.201:9150 with TLS
hephaestus[3898172]: Hermes client is ready
```

## Quicly generate self-signed certificates for TLS

They can be done by using some commands, but openssl required:
```
openssl genrsa -des3 -out hepha_pr_localhost_ca.key 2048

openssl req -x509 -new -nodes -key hepha_pr_localhost_ca.key -sha256 -days 1825 -out hepha_pr_localhost_ca.pem

openssl genrsa -out hepha_pr_localhost.key 2048

openssl req -new -sha256 -key hepha_pr_localhost.key -out hepha_pr_localhost.csr

openssl x509 -req -in hepha_pr_localhost.csr -CA my_ca.pem -CAkey my_ca.key -CAcreateserial -out hepha_pr_localhost.pem -days 1825 -sha256 -extfile hepha_pr_localhost.ext
```

Content of server.ext like:
```
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
```