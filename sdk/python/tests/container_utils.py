#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""Test container utilities for Rooch testing"""

import os
import time
import subprocess
import atexit
import socket
import requests
from typing import Optional, List, Dict, Any, Union

class RoochNodeContainer:
    """Manages a local Rooch node for testing"""
    
    def __init__(
        self,
        image: str = "ghcr.io/rooch-network/rooch:main_debug",
        network_name: str = "local",
        data_dir: str = "TMP",
        port: int = 6767,
        eth_rpc_url: Optional[str] = None,
        btc_rpc_url: Optional[str] = None,
        btc_rpc_username: Optional[str] = None,
        btc_rpc_password: Optional[str] = None,
        btc_end_block_height: Optional[int] = None,
        btc_sync_block_interval: Optional[int] = None,
        traffic_burst_size: Optional[int] = None,
        traffic_per_second: Optional[int] = None,
        local_binary_path: Optional[str] = None,
        skip_initialization: bool = False
    ):
        """Initialize Rooch node container
        
        Args:
            image: Docker image to use
            network_name: Network name
            data_dir: Data directory
            port: RPC port
            eth_rpc_url: Ethereum RPC URL
            btc_rpc_url: Bitcoin RPC URL
            btc_rpc_username: Bitcoin RPC username
            btc_rpc_password: Bitcoin RPC password
            btc_end_block_height: Bitcoin end block height
            btc_sync_block_interval: Bitcoin sync block interval
            traffic_burst_size: Traffic burst size
            traffic_per_second: Traffic per second
            local_binary_path: Path to local Rooch binary
            skip_initialization: Skip initialization steps
        """
        self.image = image
        self.network_name = network_name
        self.data_dir = data_dir
        self.port = port
        self.eth_rpc_url = eth_rpc_url
        self.btc_rpc_url = btc_rpc_url
        self.btc_rpc_username = btc_rpc_username
        self.btc_rpc_password = btc_rpc_password
        self.btc_end_block_height = btc_end_block_height
        self.btc_sync_block_interval = btc_sync_block_interval
        self.traffic_burst_size = traffic_burst_size
        self.traffic_per_second = traffic_per_second
        self.local_binary_path = local_binary_path
        self.skip_initialization = skip_initialization
        
        self.container_id = None
        self.mapped_port = None
        
    def build_server_start_command(self) -> str:
        """Build the server start command string with all options"""
        cmd = '/rooch/rooch server start'
        
        cmd += f" -n {self.network_name}"
        cmd += f" -d {self.data_dir}"
        cmd += f" --port {self.port}"
        
        if self.eth_rpc_url:
            cmd += f" --eth-rpc-url {self.eth_rpc_url}"
            
        if self.btc_rpc_url:
            cmd += f" --btc-rpc-url {self.btc_rpc_url}"
            
        if self.btc_rpc_username:
            cmd += f" --btc-rpc-username {self.btc_rpc_username}"
            
        if self.btc_rpc_password:
            cmd += f" --btc-rpc-password {self.btc_rpc_password}"
            
        if self.btc_end_block_height is not None:
            cmd += f" --btc-end-block-height {self.btc_end_block_height}"
            
        if self.btc_sync_block_interval is not None:
            cmd += f" --btc-sync-block-interval {self.btc_sync_block_interval}"
            
        if self.traffic_per_second is not None:
            cmd += f" --traffic-per-second {self.traffic_per_second}"
            
        if self.traffic_burst_size is not None:
            cmd += f" --traffic-burst-size {self.traffic_burst_size}"
            
        return cmd
        
    def start(self) -> str:
        """Start the container and return the mapped URL"""
        # Create full command
        server_start_cmd = self.build_server_start_command()
        
        if self.skip_initialization:
            full_command = server_start_cmd
        else:
            full_command = f"/rooch/rooch init --skip-password && \
                /rooch/rooch env switch --alias local && \
                {server_start_cmd}"
        
        # Prepare docker run command
        docker_cmd = [
            "docker", "run", "-d", "--rm",
            "-p", f"{self.port}:{self.port}",
            "--entrypoint", "/bin/bash",
            self.image,
            "-c", full_command
        ]
        
        # Add volume mounts if using local binary
        if self.local_binary_path:
            if not os.path.exists(self.local_binary_path):
                raise ValueError(f"Local Rooch binary not found at {self.local_binary_path}")
            docker_cmd.extend(["-v", f"{self.local_binary_path}:/rooch/rooch"])
        
        # Start container
        result = subprocess.run(docker_cmd, capture_output=True, text=True)
        if result.returncode != 0:
            raise RuntimeError(f"Failed to start container: {result.stderr}")
        
        self.container_id = result.stdout.strip()
        self.mapped_port = self.port  # In this simple case, we're using host port binding
        
        # Register cleanup on exit
        atexit.register(self.stop)
        
        # Wait for server to be ready
        self._wait_for_server()
        
        # Return the URL for RPC client
        return f"http://localhost:{self.mapped_port}/v1/jsonrpc"
    
    def _wait_for_server(self, timeout=120, interval=1):
        """Wait for the server to be ready"""
        endpoint = f"http://localhost:{self.mapped_port}/v1/jsonrpc"
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            # Check if server is ready by tailing logs
            logs_cmd = ["docker", "logs", self.container_id]
            result = subprocess.run(logs_cmd, capture_output=True, text=True)
            if "JSON-RPC HTTP Server start listening" in result.stdout:
                return
            
            # Alternative: try to connect to the server
            try:
                response = requests.post(
                    endpoint, 
                    json={
                        "jsonrpc": "2.0",
                        "method": "rooch_getInfo",
                        "params": [],
                        "id": 1
                    },
                    timeout=2
                )
                if response.status_code == 200:
                    return
            except:
                pass
                
            time.sleep(interval)
            
        raise TimeoutError(f"Server did not start within {timeout} seconds")
    
    def stop(self):
        """Stop the container"""
        if self.container_id:
            try:
                subprocess.run(["docker", "stop", self.container_id], capture_output=True)
                # Unregister the atexit handler
                atexit.unregister(self.stop)
                self.container_id = None
            except Exception as e:
                print(f"Error stopping container: {e}")