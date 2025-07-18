#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

"""
Example of using session management with the Rooch Python SDK.
This demonstrates how to create and use sessions for dApps.
"""

import asyncio
import json
from typing import Dict, Any, List

from rooch.client.client import RoochClient
from rooch.transport import RoochEnvironment
from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import Signer
from rooch.session.session import SessionClient, SessionArgs


async def create_session(
    client: RoochClient,
    signer: Signer,
    app_name: str,
    app_url: str,
    scopes: List[str],
) -> Dict[str, Any]:
    """Create a new session for a dApp
    
    Args:
        client: Rooch client
        signer: Transaction signer
        app_name: Name of the application
        app_url: URL of the application
        scopes: List of function IDs the session is authorized to call
        
    Returns:
        Session information
    """
    print(f"\n=== Creating session for app {app_name} ===")
    
    try:
        # Create a session client
        session_client = SessionClient(client)
        
        # Create session arguments
        session_args = SessionArgs(
            app_name=app_name,
            app_url=app_url,
            scopes=scopes
        )
        
        # Create the session
        session = await session_client.create_session(
            session_args=session_args,
            signer=signer
        )
        
        print(f"Session created: {json.dumps(session, indent=2)}")
        return session
    except Exception as e:
        print(f"Error creating session: {e}")
        return {}


async def use_session_for_call(
    client: RoochClient,
    session: Dict[str, Any],
    function_id: str,
    type_args: List[str] = None,
    args: List[Any] = None
) -> Dict[str, Any]:
    """Use a session to execute a function call
    
    Args:
        client: Rooch client
        session: Session information
        function_id: Function ID to call
        type_args: Type arguments
        args: Function arguments
        
    Returns:
        Transaction result
    """
    print(f"\n=== Using session to call {function_id} ===")
    
    try:
        # Create a session client
        session_client = SessionClient(client)
        
        # Create a session signer
        session_signer = session_client.create_session_signer(session)
        
        # Execute the function call using the session
        result = await client.execute_move_call(
            signer=session_signer,
            function_id=function_id,
            type_args=type_args if type_args else [],
            args=args if args else [],
            max_gas_amount=10_000_000
        )
        
        print(f"Function call result: {json.dumps(result, indent=2)}")
        return result
    except Exception as e:
        print(f"Error executing function with session: {e}")
        return {}


async def list_sessions(client: RoochClient, owner_address: str) -> List[Dict[str, Any]]:
    """List all sessions for an account
    
    Args:
        client: Rooch client
        owner_address: Owner account address
        
    Returns:
        List of sessions
    """
    print(f"\n=== Listing sessions for {owner_address} ===")
    
    try:
        # Create a session client
        session_client = SessionClient(client)
        
        # List sessions
        sessions = await session_client.list_sessions(owner_address)
        
        print(f"Found {len(sessions)} sessions")
        for i, session in enumerate(sessions):
            print(f"Session {i+1}: {session.get('session_id', 'unknown')}")
            print(f"  App Name: {session.get('app_name', 'unknown')}")
            print(f"  App URL: {session.get('app_url', 'unknown')}")
            print(f"  Scopes: {session.get('scopes', [])}")
        
        return sessions
    except Exception as e:
        print(f"Error listing sessions: {e}")
        return []


async def revoke_session(
    client: RoochClient,
    signer: Signer,
    session_id: str
) -> Dict[str, Any]:
    """Revoke a session
    
    Args:
        client: Rooch client
        signer: Transaction signer
        session_id: ID of the session to revoke
        
    Returns:
        Transaction result
    """
    print(f"\n=== Revoking session {session_id} ===")
    
    try:
        # Create a session client
        session_client = SessionClient(client)
        
        # Revoke the session
        result = await session_client.revoke_session(
            signer=signer,
            session_id=session_id
        )
        
        print(f"Session revoked: {json.dumps(result, indent=2)}")
        return result
    except Exception as e:
        print(f"Error revoking session: {e}")
        return {}


async def main() -> None:
    """Main function"""
    try:
        # Connect to Rooch local node
        async with RoochClient(RoochEnvironment.LOCAL) as client:
            print("=== Connected to Rooch node ===")
            
            # Generate a key pair
            # In a real application, you would use a private key from a secure source
            keypair = KeyPair.generate()
            signer = Signer(keypair)
            
            address = signer.get_address()
            print(f"Using address: {address}")
            
            # Create a session for a dApp
            session = await create_session(
                client,
                signer,
                app_name="Example dApp",
                app_url="https://example.com",
                scopes=["0x3::empty::empty_with_signer"]  # Example scope
            )
            
            # Check if session creation was successful
            if session and "session_id" in session:
                # Use the session to execute a function call
                await use_session_for_call(
                    client,
                    session,
                    function_id="0x3::empty::empty_with_signer",
                    type_args=[],
                    args=[]
                )
                
                # List all sessions for the account
                sessions = await list_sessions(client, address)
                
                # Revoke the session
                if sessions and len(sessions) > 0:
                    session_id = sessions[0].get("session_id")
                    if session_id:
                        await revoke_session(client, signer, session_id)
                        
                        # Verify session was revoked by listing sessions again
                        await list_sessions(client, address)
    
    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    asyncio.run(main())