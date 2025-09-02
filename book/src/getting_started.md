# Getting Started

This guide will help you get up and running with Bevygap quickly.

## What You'll Need

Before starting, make sure you have:

- A Bevy game using [Lightyear](https://crates.io/crates/lightyear) for networking
- An [Edgegap](https://edgegap.com) account (free tier is fine for testing)
- A server with a public IP address for running NATS
- Basic familiarity with Docker and container deployment

## Quick Overview

Bevygap consists of several components that work together:

1. **NATS Server** - Message broker and key-value store (you host this)
2. **Matchmaker Service** - Handles player requests and manages Edgegap sessions
3. **HTTP Frontend** - Web interface that game clients connect to
4. **Game Plugins** - Bevy plugins for your client and server
5. **Edgegap Integration** - Automatically deploys game servers on demand

## Setup Process

The basic setup process is:

1. **[Set up NATS Server](./installation/nats.md)** - Deploy a public NATS server with TLS
2. **[Configure Bevygap](./installation/bevygap_nats.md)** - Set up environment variables for NATS connection
3. **[Configure Edgegap](./installation/edgegap.md)** - Set up your Edgegap application and deployment pipeline
4. **[Deploy Matchmaker](./installation/matchmaker_services.md)** - Run the matchmaking services
5. **[Integrate with Your Game](./installation/game_client.md)** - Add Bevygap plugins to your client and server

## Development Workflow

Once set up, your typical workflow will be:

1. **Develop Locally** - Work on your game using direct connections or local testing
2. **Test with Edgegap** - Deploy to Edgegap when you need to test the full flow
3. **Deploy Updates** - Push git tags to trigger automated builds and deployments

## Example Project

The [bevygap-spaceships](https://github.com/RJ/bevygap-spaceships) repository provides a complete working example of a game integrated with Bevygap. It's based on the Lightyear "spaceships" example and shows how to:

- Set up client and server plugins
- Configure networking for WebTransport
- Deploy to Edgegap using GitHub Actions

> **Note**: Remember to update the dependencies in bevygap-spaceships to use `bananabit-dev/bevygap` instead of the original `RJ/bevygap`. See the [External Examples Update Guide](../EXTERNAL_EXAMPLES_UPDATE.md) for details.

## Next Steps

Ready to get started? Head to the [Installation](./installation/index.md) section to begin setting up your Bevygap deployment.

For local development without the full infrastructure, check out [Developing your Game](./development/index.md).