# PlayChannel

A reference implementation of an off-chain [state channel](https://ethereum.org/en/developers/docs/scaling/state-channels/)
built using [Anza's SVM API](https://www.anza.xyz/blog/anzas-new-svm-api).

With the release of Agave 2.0, we've decoupled the SVM API from the rest of the
runtime, which means it can be used outside the validator. This unlocks
SVM-based solutions such as sidecars, channels, rollups, and more. This project
demonstrates everything you need to know about boostrapping with this new API.

PlayChannel is a state channel designed for mini-games, enabling multiple participants to engage in gaming interactions with moves processed off-chain. Once the channel is closed, the resulting changes such as wagers, rewards or updated balances are posted to the base chain.

## Generic Architecture

PlayChannel implements a modular and generic archtecture, making it adaptable to different games. It uses a plugin-like system that allows developers to implement any game of their choice by defining game specific logic and rules.

## Game Configuration

Games can be configured using a set of predefined traits that define:

- Game Rules: Specify the logic for valid moves, winning conditions, and penalties.
- State Transitions: Define how the game state evolves with each move or interaction.
- Player Actions: Configure the types of actions players can take and how they are validated.
<!-- - Dispute Resolution: Handle conflicts by resolving disputes on-chain, ensuring fairness and integrity. -->

## Example Use Cases

1. Turn-Based Games: Implement games like chess, checkers, or card games where players take turns making moves. The state channel ensures that moves are validated off-chain, and the final result is settled on-chain.
