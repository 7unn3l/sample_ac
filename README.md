# setup

1) setup k8s kind
install kind (refer to website)
kind create cluster

2) certs n stuff

To generate a self-signed Certificate Authority (CA) and a certificate, you can use OpenSSL, a widely-used and versatile tool for working with certificates. Follow these steps to generate your CA and certificate:

1. Generate a private key for the CA:

```sh
openssl genrsa -out ca.key 2048
```

This command creates a 2048-bit RSA private key and saves it to a file called `ca.key`.

2. Create a self-signed CA certificate:

```sh
openssl req -x509 -new -nodes -key ca.key -subj "/CN=My Custom CA" -days 3650 -out ca.crt
```

This command generates a self-signed X.509 certificate using the private key from the previous step. The certificate will be valid for 10 years (3650 days) and will be saved to a file called `ca.crt`. The `/CN=My Custom CA` part sets the common name (CN) of the certificate.

3. Generate a private key for the certificate:

```sh
openssl genrsa -out server.key 2048
```

This command creates a 2048-bit RSA private key for the server certificate and saves it to a file called `server.key`.

4. Create a Certificate Signing Request (CSR) for the server certificate:

```sh
openssl req -new -key server.key -subj "/CN=my-admission-controller" -out server.csr
```

This command generates a CSR using the server's private key from the previous step. The `/CN=my-admission-controller` part sets the common name (CN) of the server certificate. The CSR will be saved to a file called `server.csr`.

5. Sign the CSR with the CA:

```sh
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out server.crt -days 3650 -sha256
```

This command signs the CSR using the CA certificate and private key, creating a server certificate that is valid for 10 years (3650 days) and saved to a file called `server.crt`.

Now you have a self-signed CA (`ca.crt` and `ca.key`) and a server certificate (`server.crt` and `server.key`). If you're using these certificates for a Kubernetes webhook, you'll need to base64-encode the CA certificate for the `caBundle` field in the `ValidatingWebhookConfiguration` resource. You can do that with the following command:

```sh
cat ca.crt | base64 | tr -d '\n'
```

kubectl apply -f validatingwebhook.yaml

map sample-ac.local in hosts file (local ip / not localhosts) so kind
in docker can access it

# Kubectl interaction

kubectl operates on a single cluster at a time

get all clusters:
kubectl config  get-contexts

first deploy webhook
kubectl apply -f validatingwebhook.yaml

then try to deploy example pod
kubectl apply -f samples/sample_createpod.yaml
