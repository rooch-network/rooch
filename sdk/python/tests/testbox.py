#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""TestBox for Rooch testing"""

import os
import sys
import time
import shutil
import tempfile
import subprocess
import json
from typing import Optional, List, Dict, Union, Any, Tuple
import logging
from pathlib import Path

from .container_utils import RoochNodeContainer

class TestBox:
    """
    TestBox provides a testing environment for Rooch tests.
    It can run a Rooch node in either local binary mode or container mode.
    """
    
    def __init__(
        self,
        mode: str = "local",
        network_name: str = "local",
        port: int = 6767,
        local_binary_path: Optional[str] = None,
        container_image: str = "ghcr.io/rooch-network/rooch:main_debug",
        data_dir: Optional[str] = None,
        eth_rpc_url: Optional[str] = None,
        btc_rpc_url: Optional[str] = None,
        btc_rpc_username: Optional[str] = None,
        btc_rpc_password: Optional[str] = None,
        btc_end_block_height: Optional[int] = None,
        btc_sync_block_interval: Optional[int] = None,
        log_level: str = "info"
    ):
        """Initialize TestBox
        
        Args:
            mode: 'local' to use local binary or 'container' to use Docker
            network_name: Network name
            port: RPC port
            local_binary_path: Path to local Rooch binary
            container_image: Docker image to use (only for container mode)
            data_dir: Data directory (if None, a temporary directory is created)
            eth_rpc_url: Ethereum RPC URL
            btc_rpc_url: Bitcoin RPC URL
            btc_rpc_username: Bitcoin RPC username
            btc_rpc_password: Bitcoin RPC password
            btc_end_block_height: Bitcoin end block height
            btc_sync_block_interval: Bitcoin sync block interval
            log_level: Logging level
        """
        self.mode = mode
        if mode not in ["local", "container"]:
            raise ValueError(f"Invalid mode: {mode}. Must be 'local' or 'container'")
        
        self.network_name = network_name
        self.port = port
        self.eth_rpc_url = eth_rpc_url
        self.btc_rpc_url = btc_rpc_url
        self.btc_rpc_username = btc_rpc_username
        self.btc_rpc_password = btc_rpc_password
        self.btc_end_block_height = btc_end_block_height
        self.btc_sync_block_interval = btc_sync_block_interval
        self.log_level = log_level
        
        # Set up local binary path
        if local_binary_path:
            self.local_binary_path = local_binary_path
        else:
            # Try to find rooch in PATH
            self.local_binary_path = shutil.which("rooch")
            if not self.local_binary_path and mode == "local":
                raise ValueError("No local binary path provided and 'rooch' not found in PATH")
        
        self.container_image = container_image
        
        # Create temporary directory if no data_dir provided
        self._temp_dir = None
        if data_dir:
            self.data_dir = data_dir
        else:
            self._temp_dir = tempfile.TemporaryDirectory()
            self.data_dir = self._temp_dir.name
        
        # Initialize members
        self.url = None
        self.process = None
        self.container = None
        
        # Setup logging
        logging.basicConfig(
            level=getattr(logging, log_level.upper(), logging.INFO),
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
        self.logger = logging.getLogger("TestBox")
    
    def __del__(self):
        """Clean up resources on deletion"""
        self.stop()
        if self._temp_dir:
            self._temp_dir.cleanup()
    
    def start(self) -> str:
        """Start the Rooch node and return the RPC URL"""
        if self.url:
            self.logger.warning("TestBox is already started")
            return self.url
        
        if self.mode == "local":
            return self._start_local()
        else:  # container mode
            return self._start_container()
    
    def _start_local(self) -> str:
        """Start a local binary Rooch node"""
        # Create data directory if it doesn't exist
        os.makedirs(self.data_dir, exist_ok=True)
        
        # Initialize if needed (first run)
        init_done_file = os.path.join(self.data_dir, ".init_done")
        if not os.path.exists(init_done_file):
            init_cmd = [
                self.local_binary_path, "init", 
                "--data-dir", self.data_dir,
                "--skip-password"
            ]
            
            self.logger.info(f"Initializing Rooch: {' '.join(init_cmd)}")
            result = subprocess.run(init_cmd, capture_output=True, text=True)
            if result.returncode != 0:
                raise RuntimeError(f"Failed to initialize Rooch: {result.stderr}")
            
            # Switch to local network
            switch_cmd = [
                self.local_binary_path, "env", "switch",
                "--data-dir", self.data_dir,
                "--alias", self.network_name
            ]
            
            self.logger.info(f"Switching to network {self.network_name}: {' '.join(switch_cmd)}")
            result = subprocess.run(switch_cmd, capture_output=True, text=True)
            if result.returncode != 0:
                raise RuntimeError(f"Failed to switch network: {result.stderr}")
            
            # Mark initialization as done
            with open(init_done_file, 'w') as f:
                f.write("init done")
        
        # Build server start command
        cmd = [
            self.local_binary_path, "server", "start",
            "--data-dir", self.data_dir,
            "-n", self.network_name,
            "--port", str(self.port)
        ]
        
        # Add optional parameters
        if self.eth_rpc_url:
            cmd.extend(["--eth-rpc-url", self.eth_rpc_url])
        
        if self.btc_rpc_url:
            cmd.extend(["--btc-rpc-url", self.btc_rpc_url])
        
        if self.btc_rpc_username:
            cmd.extend(["--btc-rpc-username", self.btc_rpc_username])
        
        if self.btc_rpc_password:
            cmd.extend(["--btc-rpc-password", self.btc_rpc_password])
        
        if self.btc_end_block_height is not None:
            cmd.extend(["--btc-end-block-height", str(self.btc_end_block_height)])
        
        if self.btc_sync_block_interval is not None:
            cmd.extend(["--btc-sync-block-interval", str(self.btc_sync_block_interval)])
        
        # Start the process
        self.logger.info(f"Starting Rooch server: {' '.join(cmd)}")
        self.process = subprocess.Popen(
            cmd, 
            stdout=subprocess.PIPE, 
            stderr=subprocess.PIPE,
            text=True
        )
        
        # Wait for the server to be ready
        self._wait_for_local_server()
        
        # Set URL
        self.url = f"http://localhost:{self.port}/v1/jsonrpc"
        self.logger.info(f"Rooch server started at {self.url}")
        
        return self.url
    
    def _wait_for_local_server(self, timeout=60, interval=1):
        """Wait for the local server to be ready"""
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            if self.process.poll() is not None:
                # Process exited
                stdout, stderr = self.process.communicate()
                raise RuntimeError(f"Local Rooch server exited unexpectedly: \nSTDOUT: {stdout}\nSTDERR: {stderr}")
            
            # Check if we can read from the process output
            line = self.process.stdout.readline()
            if line and "JSON-RPC HTTP Server start listening" in line:
                return
            
            time.sleep(interval)
        
        # Timeout reached, kill the process
        self.process.terminate()
        raise TimeoutError("Timed out waiting for local Rooch server to start")
    
    def _start_container(self) -> str:
        """Start a Docker container Rooch node"""
        self.container = RoochNodeContainer(
            image=self.container_image,
            network_name=self.network_name,
            data_dir=self.data_dir,
            port=self.port,
            eth_rpc_url=self.eth_rpc_url,
            btc_rpc_url=self.btc_rpc_url,
            btc_rpc_username=self.btc_rpc_username,
            btc_rpc_password=self.btc_rpc_password,
            btc_end_block_height=self.btc_end_block_height,
            btc_sync_block_interval=self.btc_sync_block_interval,
            local_binary_path=self.local_binary_path if self.mode == "local" else None
        )
        
        self.logger.info("Starting Rooch container")
        self.url = self.container.start()
        self.logger.info(f"Rooch server started in container at {self.url}")
        
        return self.url
    
    def stop(self) -> None:
        """Stop the Rooch node"""
        if self.mode == "local" and self.process:
            self.logger.info("Stopping local Rooch server")
            self.process.terminate()
            try:
                self.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.process.kill()
            self.process = None
        
        if self.mode == "container" and self.container:
            self.logger.info("Stopping Rooch container")
            self.container.stop()
            self.container = None
        
        self.url = None
    
    def exec_command(self, cmd: List[str], cwd: Optional[str] = None) -> Tuple[int, str, str]:
        """Execute a Rooch CLI command and return the result
        
        Args:
            cmd: Command to execute (without the 'rooch' prefix)
            cwd: Current working directory for the command
            
        Returns:
            Tuple of (returncode, stdout, stderr)
        """
        full_cmd = [self.local_binary_path if self.mode == "local" else "rooch"]
        full_cmd.extend(cmd)
        
        # Add data directory if local mode
        if self.mode == "local":
            data_dir_added = False
            for i, arg in enumerate(cmd):
                if arg in ["--data-dir", "-d"]:
                    data_dir_added = True
                    break
            
            if not data_dir_added:
                full_cmd.extend(["--data-dir", self.data_dir])
        
        self.logger.debug(f"Executing command: {' '.join(full_cmd)}")
        result = subprocess.run(full_cmd, capture_output=True, text=True, cwd=cwd)
        
        return result.returncode, result.stdout, result.stderr
    
    def publish_package(self, package_path: str) -> str:
        """Publish a Move package and return the transaction hash
        
        Args:
            package_path: Path to the Move package
            
        Returns:
            Transaction hash
        """
        cmd = ["move", "publish", "--skip-fetch-latest-git-deps", "--json-output", package_path]
        returncode, stdout, stderr = self.exec_command(cmd)
        
        if returncode != 0:
            raise RuntimeError(f"Failed to publish package: {stderr}")
        
        # Parse transaction hash from JSON output
        result = json.loads(stdout)
        return result.get("transaction_hash")
    
    def run_function(
        self, 
        function_id: str, 
        type_args: Optional[List[str]] = None,
        args: Optional[List[str]] = None
    ) -> str:
        """Run a Move function and return the transaction hash
        
        Args:
            function_id: Function ID in the format 'address::module::function'
            type_args: List of type arguments
            args: List of arguments
            
        Returns:
            Transaction hash
        """
        cmd = ["move", "run", "--json-output", function_id]
        
        if type_args:
            cmd.extend(["--type-args", ",".join(type_args)])
        
        if args:
            cmd.extend(["--args", " ".join(args)])
        
        returncode, stdout, stderr = self.exec_command(cmd)
        
        if returncode != 0:
            raise RuntimeError(f"Failed to run function: {stderr}")
        
        # Parse transaction hash from JSON output
        result = json.loads(stdout)
        return result.get("transaction_hash")
    
    def view_function(
        self, 
        function_id: str, 
        type_args: Optional[List[str]] = None,
        args: Optional[List[str]] = None
    ) -> Any:
        """View a Move function and return the result
        
        Args:
            function_id: Function ID in the format 'address::module::function'
            type_args: List of type arguments
            args: List of arguments
            
        Returns:
            Function result
        """
        cmd = ["move", "view", "--json-output", function_id]
        
        if type_args:
            cmd.extend(["--type-args", ",".join(type_args)])
        
        if args:
            cmd.extend(["--args", " ".join(args)])
        
        returncode, stdout, stderr = self.exec_command(cmd)
        
        if returncode != 0:
            raise RuntimeError(f"Failed to view function: {stderr}")
        
        # Parse result from JSON output
        result = json.loads(stdout)
        return result.get("result")