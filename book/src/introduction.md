# Introduction

> **Note**: This documentation is for the maintained fork of Bevygap at [bananabit-dev/bevygap](https://github.com/bananabit-dev/bevygap), which is based on the original work by RJ at [RJ/bevygap](https://github.com/RJ/bevygap). This fork includes updates, bug fixes, and continued development.

Bevygap is a suite of tools to help you get your (bevy+lightyear) game running on Edgegap, so that servers are spun up and down in response to player demand, in appropriate locations. When players click "connect", the system either picks an existing server nearby, or spins up a new one, which takes just a few seconds.

### Scaling and costs

It **scales down to zero** pretty well - Edgegap can be configured to terminate server instances with no players in the last 10 minutes, and you are only billed for server uptime. There's a small static cost to running your NATS server and matchmaking service. I'm running mine on the same linux server that hosts my personal website.

In theory, it **will scale up** pretty well too. Edgegap will keep launching new servers for you and directing new players to them. Nice problem to have, and not one i've encountered yet :)

### Bevygap components

* A bevy plugin for your gameserver, `bevygap_server_plugin`
* A bevy plugin for your game clients, `bevygap_client_plugin`
* A matchmaker service that talks to the Edgegap API, `bevygap_matchmaker`
* A webserver frontend for your matchmaker service, that `bevygap_client_plugin` talks to: `bevygap_httpd` 
* A shared crate, `bevygap_shared`, used to connect to the NATS message bus.
* An example game, `bevygap-spaceships`, which is deployable to Edgegap using all of the above.

![Bevygap Architecture](assets/bevygap-20241105.png)

## Dev, test, deploy cycle

### Local Development
For day-to-day development, you can work entirely locally without Docker or Edgegap:

1. **Direct Connection**: Configure your game to bypass Bevygap and connect directly between client and server
2. **Local NATS**: Set up a local NATS server for testing the messaging system
3. **Mock Matchmaker**: Use simplified matchmaking logic for local testing

### Testing with Edgegap
When you're ready to test the full deployment pipeline:

1. **Configure Edgegap**: Set up your Edgegap application and environment variables
2. **Deploy Gameserver**: Push a git tag or manually trigger the GitHub Action to build and deploy your gameserver container
3. **Test Matchmaking**: Connect clients through the full matchmaking flow

### Production Deployment
For production deployment:

1. **Tag Release**: Create a git tag (e.g., `v1.0.0`) to trigger the build pipeline
2. **GitHub Actions**: The automated workflow builds your server container and pushes it to Edgegap's container registry
3. **Edgegap Deployment**: Edgegap automatically deploys your server when players request games
4. **Monitoring**: Monitor server usage and costs through the Edgegap dashboard

The build process typically takes 10-15 minutes and can be optimized with Docker and GitHub caching strategies.
