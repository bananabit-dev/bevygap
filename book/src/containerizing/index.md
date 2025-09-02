# Containerizing Services

This section covers how to containerize and deploy the Bevygap services for production use.

## Available Services

Bevygap includes several services that can be containerized and deployed:

### Matchmaker Service (`bevygap_matchmaker`)
The core matchmaking service that handles player requests and manages Edgegap sessions.

### Matchmaker HTTP Service (`bevygap_matchmaker_httpd`)  
Web frontend that provides HTTP/WebSocket endpoints for clients to connect to the matchmaker.

### Webhook Sink (`bevygap_webhook_sink`)
Service for handling webhooks from Edgegap about deployment status changes.

## GitHub Actions Workflows

The repository includes automated workflows for building and publishing Docker images:

- **`publish-matchmaker.yaml`** - Builds and publishes the matchmaker service
- **`publish-matchmaker-httpd.yaml`** - Builds and publishes the HTTP frontend  
- **`publish-webhook-sink.yaml`** - Builds and publishes the webhook service
- **`publish-book.yaml`** - Builds and publishes this documentation

## Docker Compose Setup

The repository includes a `docker-compose.yml` file for local development and testing that sets up:

- NATS server with TLS
- Traefik reverse proxy
- All Bevygap services
- Proper networking and environment configuration

To use the Docker Compose setup:

```bash
# Copy environment files
cp .matchmaker.env.example .matchmaker.env
cp edgegap.env.example edgegap.env
cp nats.env.example nats.env

# Edit the environment files with your configuration
# Then start the services
docker-compose up -d
```

## Production Deployment Considerations

### Environment Variables
Each service requires specific environment variables for:
- NATS connection details
- Edgegap API credentials  
- TLS certificate configuration

### Networking
- Services need to communicate with external NATS server
- HTTP service needs to be accessible to game clients
- Consider using a reverse proxy like Traefik for SSL termination

### Monitoring and Logging
- Services log to stdout/stderr for container log aggregation
- Consider setting up proper log aggregation and monitoring
- Monitor NATS connection health and Edgegap API usage

### Scaling
- Matchmaker services are generally stateless and can be scaled horizontally
- NATS provides the shared state layer between instances
- Consider load balancing for the HTTP frontend service