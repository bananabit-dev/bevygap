## Bevygap NATS Setup

Now that `nats-cli` can connect to your NATS server, and we know it's working, let's ensure that the bevygap code can connect too.

### TLS Certificate Verification with LetsEncrypt

Bevygap now uses LetsEncrypt certificates for secure NATS connections. LetsEncrypt certificates are automatically trusted by the system, so no additional CA certificate configuration is required.

**If you encounter TLS handshake errors**, ensure your NATS server is configured with valid LetsEncrypt certificates and that the certificates are not expired.

### Bevygap Required Environment Variables

`bevygap_matchmaker`, `bevygap_httpd`, and the gameservers (via `bevygap_server_plugin`) need to connect to NATS.

The NATS connection code in `bevygap_shared` depends on the following environment variables to set up the NATS connection.

| Variable         | Required | Description                                                                                                                                                                                                                                  |
| ---------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| NATS_HOST        | Yes      | NATS server address<br><small>eg: `nats.example.com:4222` or `1.2.3.4`</small>                                                                                                                                                               |
| NATS_USER        | Yes      | Username for NATS authentication                                                                                                                                                                                                             |
| NATS_PASSWORD    | Yes      | Password for NATS authentication                                                                                                                                                                                                             |
| NATS_INSECURE    | No       | Disable TLS entirely (any value)<br><small>**Not recommended for production**</small>                                                                                                                                                       |
| NATSRETRYCOUNT   | No       | Number of connection retry attempts<br><small>Defaults to 3. Supports IPv6/IPv4 fallback for domain names.</small>                                                                                                                          |

### Legacy Variables (Deprecated)

The following variables were used for self-signed certificates but are now deprecated since LetsEncrypt certificates are used:

| Variable         | Status | Description |
| ---------------- | ------ | ----------- |
| NATS_CA          | **Deprecated** | Path to CA root certificate<br><small>No longer needed with LetsEncrypt certificates</small> |
| NATS_CA_CONTENTS | **Deprecated** | Contents of the CA file as a string<br><small>No longer needed with LetsEncrypt certificates</small> |

### How TLS Certificate Verification Works

The Bevygap NATS client automatically handles certificate verification:

1. **LetsEncrypt certificates (recommended):** The client uses the system's trusted CA store for verification
2. **Legacy CA support:** If `NATS_CA` or `NATS_CA_CONTENTS` is set, custom CA certificates are still supported (deprecated)
3. **Insecure mode:** If `NATS_INSECURE` is set, TLS verification is disabled entirely (not recommended)

### Legacy Support: Command Line Arguments (Deprecated)

For legacy deployments that still use custom CA certificates, the old method of passing CA certificate contents as command line arguments is still supported but deprecated:

```bash
# For gameservers using bevygap_server_plugin (deprecated)
./your_gameserver --ca_contents "$(cat /path/to/rootCA.pem)"
```

**Note:** This method is deprecated since LetsEncrypt certificates are now used. The system trust store automatically handles certificate verification for trusted certificates.


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


