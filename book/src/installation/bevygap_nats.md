## Bevygap NATS Setup

Now that `nats-cli` can connect to your NATS server, and we know it's working, let's ensure that the bevygap code can connect too.

### Root CA Certificate Loading for TLS Connections

When connecting to a NATS server with self-signed certificates, the client must be able to verify the server's certificate. This requires loading the root CA certificate that was used to sign the server's certificate.

**Common Error:** If you see "unknown certificate authority" or TLS handshake errors, it means the NATS client cannot verify the server's certificate because it doesn't trust the CA that signed it.

**Solution:** Provide the root CA certificate to the Bevygap NATS client using one of the environment variables described below.

### Bevygap Required Environment Variables

`bevygap_matchmaker`, `bevygap_httpd`,and the gameservers (via `bevygap_server_plugin`) need to connect to NATS.

The NATS connection code in `bevygap_shared` depends on the following environment variables to set up the NATS connection.

| Variable         | Required | Description                                                                                                                                                                                                                                  |
| ---------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| NATS_HOST        | Yes      | NATS server address<br><small>eg: `nats.example.com:4222` or `1.2.3.4`</small>                                                                                                                                                               |
| NATS_USER        | Yes      | Username for NATS authentication                                                                                                                                                                                                             |
| NATS_PASSWORD    | Yes      | Password for NATS authentication                                                                                                                                                                                                             |
| NATS_CA          | No       | Path to CA root certificate for self-signed certs<br><small>eg: `/path/to/rootCA.pem`<br>**Use this when the CA file is already on the filesystem**</small>                                                                                |
| NATS_CA_CONTENTS | No       | Contents of the CA file as a string<br><small>**Use this when you need to pass the CA certificate contents directly**<br>Gets written to tmp file and used as NATS_CA<br><span style="color:red">255 byte limit on edgegap for ENVS<br>see note about <code>set-caroot-argument.sh</code> in 'Edgegap Setup' section</span></small> |
| NATS_INSECURE    | No       | Disable TLS entirely (any value)<br><small>**Not recommended for production**</small>                                                                                                                                                       |
| NATSRETRYCOUNT   | No       | Number of connection retry attempts<br><small>Defaults to 3. Supports IPv6/IPv4 fallback for domain names.</small>                                                                                                                          |

### How Root CA Certificate Loading Works

The Bevygap NATS client automatically handles CA certificate loading based on the environment variables:

1. **If `NATS_CA` is set:** The client loads the CA certificate from the specified file path
2. **If `NATS_CA_CONTENTS` is set:** The client writes the certificate contents to a temporary file and loads it
3. **If neither is set:** The client relies on the system's trusted CA store (suitable for LetsEncrypt certificates)
4. **If `NATS_INSECURE` is set:** TLS verification is disabled entirely (not recommended)

### Alternative Method: Command Line Arguments (Edgegap/Container Deployments)

For containerized deployments where environment variable size is limited (like Edgegap's 255-byte limit), you can pass the CA certificate contents as a command line argument:

```bash
# For gameservers using bevygap_server_plugin
./your_gameserver --ca_contents "$(cat /path/to/rootCA.pem)"
```

This method is automatically handled by the `bevygap_server_plugin` and internally sets the `NATS_CA_CONTENTS` environment variable.

**Note:** This is particularly useful for Edgegap deployments where environment variables are size-constrained, but command line arguments can handle larger certificate files (~2KB).


### Create nats.env file

Back on your local machine, in the bevygap directory, copy `nats.env.example` to `nats.env`,
and edit it with your server's IP address, nats user, nats password, and CA certificate configuration.

**Example nats.env for self-signed certificates:**
```bash
NATS_USER=matchmaker
NATS_PASSWORD=matchmaker
NATS_HOST=1.2.3.4
# Use the CA certificate file path (recommended for local development)
NATS_CA="/Users/rj/Library/Application Support/mkcert/rootCA.pem"
```

**Example nats.env for trusted certificates (LetsEncrypt):**
```bash
NATS_USER=matchmaker
NATS_PASSWORD=matchmaker
NATS_HOST=nats.yourdomain.com:4222
# No CA configuration needed - system trust store will be used
```

**Example nats.env for containerized deployments:**
```bash
NATS_USER=matchmaker
NATS_PASSWORD=matchmaker
NATS_HOST=1.2.3.4
# Pass certificate contents directly (useful for containers)
NATS_CA_CONTENTS="-----BEGIN CERTIFICATE-----
MIIBkTCB+wIJAK... (your certificate contents) ...
-----END CERTIFICATE-----"
```

Our `docker-compose.yaml` file will apply these environment variables to containers we run, but we 
also want to set them in our shell, before we run (eg) the bevygap matchmaker service using `cargo run`.

```bash
# Setting environment variables in bash, on linux/mac
export NATS_USER=....
export NATS_PASSWORD=....
# Bash trick to set them from the .env file:
set -a && . ./nats.env && set +a
```

```
# How do you do this in windows? something like this maybe:
setx NATS_USER "matchmaker"
```

Verify your environment variables are set:
```bash
$ echo $NATS_USER
matchmaker # <-- your nats username should be printed here
```

#### The final test

The `bevygap_shared` crate has an example (non-bevy) program that connects to NATS and prints a success message then exits.
This will test that your environment variables are set correctly for bevygap:

```bash
$ cargo run -p bevygap_shared --example nats
     ...compiling...
     Running `target/debug/examples/nats`
2024-11-04T09:49:23.764924Z  INFO bevygap_shared: NATS: setting up, client name: bevygap_nats_test    
2024-11-04T09:49:23.765494Z  INFO bevygap_shared: NATS: TLS is enabled    
2024-11-04T09:49:23.765498Z  INFO bevygap_shared: NATS: connecting as 'matchmaker' to 1.2.3.4    
2024-11-04T09:49:23.765512Z  INFO bevygap_shared: NATS: using self-signed CA: /Users/rj/Library/Application Support/mkcert/rootCA.pem    
2024-11-04T09:49:23.777111Z  INFO bevygap_shared: 🟢 NATS: connected OK    
2024-11-04T09:49:23.777121Z  INFO async_nats: event: connected
NATS connected OK!

```

If you made it this far, you've got a working NATS setup. Now on to the fun stuff.

### Troubleshooting TLS Certificate Issues

If you encounter errors like:

- "unknown certificate authority"
- "certificate verify failed"
- "TLS handshake failed"

Here are the steps to resolve them:

#### 1. For Self-Signed Certificates
Ensure you have the correct `rootCA.pem` file that was used to create your NATS server certificate:

```bash
# Find your CA certificate (mkcert example)
$ mkcert -CAROOT
/Users/username/Library/Application Support/mkcert

# Verify the file exists
$ ls "/Users/username/Library/Application Support/mkcert/rootCA.pem"
```

Set the environment variable:
```bash
export NATS_CA="/Users/username/Library/Application Support/mkcert/rootCA.pem"
```

#### 2. For Embedded/Container Deployments
If you can't access the filesystem or need to embed the certificate:

```bash
# Read the certificate contents
export NATS_CA_CONTENTS="$(cat /path/to/rootCA.pem)"
```

#### 3. Verify Your Setup
Test the connection with the example program:
```bash
$ cargo run -p bevygap_shared --example nats
```

#### 4. For LetsEncrypt or Trusted CAs
If your NATS server uses a certificate from a trusted CA (like LetsEncrypt), you don't need to set `NATS_CA` or `NATS_CA_CONTENTS`. The system's trusted CA store will be used automatically.

#### 5. Debug TLS Issues
Enable debug logging to see detailed TLS information:
```bash
export RUST_LOG=debug
cargo run -p bevygap_shared --example nats
```


