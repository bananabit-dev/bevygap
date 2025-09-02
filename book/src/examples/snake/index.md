# Multiplayer Snake Game

Welcome to the complete tutorial for building a multiplayer snake game using BevyGap! This example will walk you through creating a fully functional multiplayer game from scratch.

## What You'll Build

By the end of this tutorial, you'll have:

- **A multiplayer snake game** where players can join and play together
- **Real-time synchronization** between multiple clients
- **Collision detection** for snake-to-snake and snake-to-food interactions
- **Dynamic food spawning** managed by the server
- **Player scoring** and game state management
- **Local development setup** that works without external infrastructure

## Game Features

### Core Gameplay
- **Multiple players** can join the same game session
- **Snake movement** in four directions (up, down, left, right)
- **Food collection** makes snakes grow longer
- **Collision detection** with walls and other snakes
- **Score tracking** for each player

### Technical Features
- **Server-authoritative** game logic
- **Client prediction** for smooth movement
- **State replication** using Lightyear
- **Input handling** with client-side prediction
- **Local development** mode for testing

## Prerequisites

Before starting, make sure you have:

- **Rust** installed (latest stable version)
- **Basic Bevy knowledge** - familiarity with entities, components, and systems
- **Understanding of networking concepts** - client/server architecture basics

## Tutorial Structure

This tutorial is organized into several chapters:

1. **[Project Setup](./setup.md)** - Create the project structure and dependencies
2. **[Shared Components](./shared.md)** - Define common game entities and messages
3. **[Server Implementation](./server.md)** - Build the authoritative game server
4. **[Client Implementation](./client.md)** - Create the game client with rendering
5. **[Running the Game](./running.md)** - Test and play your multiplayer snake game

Each chapter builds upon the previous one, so it's recommended to follow them in order.

## Architecture Overview

The game follows a **client-server architecture**:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client 1  │    │   Client 2  │    │   Client N  │
│             │    │             │    │             │
│ • Input     │    │ • Input     │    │ • Input     │
│ • Rendering │    │ • Rendering │    │ • Rendering │
│ • Prediction│    │ • Prediction│    │ • Prediction│
└─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                    ┌─────────────┐
                    │   Server    │
                    │             │
                    │ • Game Logic│
                    │ • Physics   │
                    │ • State Mgmt│
                    │ • Authority │
                    └─────────────┘
```

- **Server**: Manages authoritative game state, processes input, handles collisions
- **Clients**: Handle input, render graphics, predict movement for smooth gameplay
- **BevyGap**: Manages matchmaking and connection setup
- **Lightyear**: Handles low-level networking and state synchronization

Ready to start? Let's begin with [Project Setup](./setup.md)!