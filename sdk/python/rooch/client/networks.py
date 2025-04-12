#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

from enum import Enum
from typing import Dict, Optional


class RoochNetwork(str, Enum):
    """Predefined Rooch networks"""
    
    MAINNET = "mainnet"
    TESTNET = "testnet"
    LOCALNET = "localnet"
    DEV = "dev"


class NetworkConfig:
    """Configuration for a Rooch network"""
    
    def __init__(
        self,
        name: str,
        http_url: str,
        ws_url: Optional[str] = None,
        explorer_url: Optional[str] = None,
        chain_id: Optional[int] = None,
        description: Optional[str] = None,
    ):
        """
        Args:
            name: Network name
            http_url: HTTP URL for the network's API
            ws_url: WebSocket URL for the network's API (if available)
            explorer_url: URL for the network's block explorer (if available)
            chain_id: Chain ID for the network (if known)
            description: Human-readable description of the network
        """
        self.name = name
        self.http_url = http_url
        self.ws_url = ws_url
        self.explorer_url = explorer_url
        self.chain_id = chain_id
        self.description = description


# Predefined network configurations
NETWORKS: Dict[RoochNetwork, NetworkConfig] = {
    RoochNetwork.MAINNET: NetworkConfig(
        name="mainnet",
        http_url="https://seed.rooch.network",
        ws_url="wss://seed.rooch.network",
        explorer_url="https://explorer.rooch.network",
        description="Rooch Mainnet"
    ),
    RoochNetwork.TESTNET: NetworkConfig(
        name="testnet",
        http_url="https://test-seed.rooch.network",
        ws_url="wss://test-seed.rooch.network",
        explorer_url="https://test-explorer.rooch.network",
        description="Rooch Testnet"
    ),
    RoochNetwork.DEV: NetworkConfig(
        name="dev",
        http_url="https://dev-seed.rooch.network",
        ws_url="wss://dev-seed.rooch.network",
        explorer_url="https://dev-explorer.rooch.network",
        description="Rooch Dev Network"
    ),
    RoochNetwork.LOCALNET: NetworkConfig(
        name="localnet",
        http_url="http://localhost:50051",
        ws_url="ws://localhost:50052",
        description="Rooch Local Development Network"
    ),
}


def get_network_config(network: RoochNetwork | str) -> NetworkConfig:
    """Get configuration for a predefined network
    
    Args:
        network: Network name or RoochNetwork enum
        
    Returns:
        Network configuration
        
    Raises:
        ValueError: If network is not recognized
    """
    if isinstance(network, str):
        try:
            network = RoochNetwork(network)
        except ValueError:
            raise ValueError(f"Unknown network: {network}")
    
    if network not in NETWORKS:
        raise ValueError(f"Unknown network: {network}")
    
    return NETWORKS[network]


def create_custom_network(
    name: str,
    http_url: str,
    ws_url: Optional[str] = None,
    explorer_url: Optional[str] = None,
    chain_id: Optional[int] = None,
    description: Optional[str] = None,
) -> NetworkConfig:
    """Create a custom network configuration
    
    Args:
        name: Network name
        http_url: HTTP URL for the network's API
        ws_url: WebSocket URL for the network's API (if available)
        explorer_url: URL for the network's block explorer (if available)
        chain_id: Chain ID for the network (if known)
        description: Human-readable description of the network
        
    Returns:
        Custom network configuration
    """
    return NetworkConfig(
        name=name,
        http_url=http_url,
        ws_url=ws_url,
        explorer_url=explorer_url,
        chain_id=chain_id,
        description=description or f"Custom network: {name}"
    )