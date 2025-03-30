# Rooch Anomalies

## Overview

Rooch Anomalies is a library for tracking and handling transaction anomalies in the Rooch blockchain system. It provides
tools to identify, store, and manage different types of transaction anomalies that may occur during blockchain
operation.

## Features

- **Transaction Anomaly Tracking**: Manage various types of transaction anomalies including:
    - Duplicate transaction hashes
    - Transactions with missing execution information
    - Transactions that should trigger accumulator reversion

- **Persistence**: Save and load anomaly data in various formats:
    - Binary serialization (using BCS)
    - JSON serialization (for Human-readable)

- **Static Anomalies**: Access to pre-defined static