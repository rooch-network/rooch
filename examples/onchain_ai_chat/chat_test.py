#!/usr/bin/env python3

import subprocess
import json
import time
from typing import List, Optional, Dict

class RoochChatTester:
    def __init__(self):
        # Get accounts from rooch
        self.accounts = self._get_accounts()
        # Use the default account as admin, and other accounts as users
        self.admin = self.accounts["default"]["hex_address"]
        self.user1 = self.accounts["account0"]["hex_address"]
        self.user2 = self.accounts["account1"]["hex_address"]
        
    def _get_accounts(self) -> Dict:
        """Get accounts from rooch command"""
        try:
            result = subprocess.run(
                ["rooch", "account", "list", "--json"],
                capture_output=True,
                text=True,
                check=True
            )
            return json.loads(result.stdout)
        except subprocess.CalledProcessError as e:
            print(f"Error getting accounts: {e.stderr}")
            raise e
        except json.JSONDecodeError as e:
            print(f"Error parsing account list JSON: {e}")
            raise e

    def run_command(self, command: List[str]) -> Optional[dict]:
        """Run a rooch command and return the JSON output if any"""
        try:
            # Print the complete command
            print(f"\nExecuting command: {' '.join(command)}")
            result = subprocess.run(command, capture_output=True, text=True, check=True)
            if result.stdout:
                print(f"Command output: {result.stdout}")
            if result.stdout and '{' in result.stdout:
                json_result = json.loads(result.stdout)
                # Check if transaction failed
                if 'output' in json_result and 'status' in json_result['output']:
                    status = json_result['output']['status']
                    if status.get('type') == 'moveabort':
                        raise Exception(f"Transaction failed: {status}")
                return json_result
            return None
        except subprocess.CalledProcessError as e:
            print(f"Error running command: {' '.join(command)}")
            print(f"Error output: {e.stderr}")
            raise e

    def create_room(self, account: str, title: str, is_public: bool) -> str:
        """Create a new chat room and return its object ID"""
        # Replace spaces with underscores in title
        safe_title = title.replace(" ", "_")
        command = [
            "rooch", "move", "run",
            "--sender", account,
            "--function", f"{self.admin}::room::create_room_entry",
            "--args", f"string:{safe_title}",
            "--args", f"bool:{str(is_public).lower()}",
            "--json"  # Add json flag to get structured output
        ]
        result = self.run_command(command)
        if result and 'output' in result:
            changes = result['output'].get('changeset', {}).get('changes', [])
            for change in changes:
                metadata = change.get('metadata', {})
                if metadata.get('object_type', '').endswith('::room::Room'):
                    return metadata.get('id')
        return None

    def send_message(self, account: str, room_id: str, message: str):
        """Send a message to a room"""
        # Replace spaces with underscores in message
        safe_message = message.replace(" ", "_")
        command = [
            "rooch", "move", "run",
            "--sender", account,
            "--function", f"{self.admin}::room::send_message_entry",
            "--args", f"object:{room_id}",
            "--args", f"string:{safe_message}",
            "--json"  # Add json flag to get structured output
        ]
        self.run_command(command)

    def add_member(self, account: str, room_id: str, member: str):
        """Add a member to a private room"""
        command = [
            "rooch", "move", "run", "--sender", account,
            "--function", f"{self.admin}::room::add_member_entry",
            "--args", f"object:{room_id}", 
            "--args", f"address:{member}",
            "--json"  # Add json flag to get structured output
        ]
        self.run_command(command)

def main():
    tester = RoochChatTester()
    
    print("=== Testing Chat Room Contract ===")
    print(f"Using accounts:")
    print(f"Admin: {tester.admin}")
    print(f"User1: {tester.user1}")
    print(f"User2: {tester.user2}")
    
    # Test 1: Create public room
    print("\n1. Creating public room...")
    public_room_id = tester.create_room(tester.admin, "Public_Room", True)
    print(f"Public room created with ID: {public_room_id}")
    
    # Test 2: Send message to public room
    print("\n2. Sending message to public room...")
    tester.send_message(tester.user1, public_room_id, "Hello,_public_room!")
    print("Message sent successfully")
    
    # Test 3: Create private room
    print("\n3. Creating private room...")
    private_room_id = tester.create_room(tester.admin, "Private Room", False)
    print(f"Private room created with ID: {private_room_id}")
    
    # Test 4: Add member to private room
    print("\n4. Adding member to private room...")
    tester.add_member(tester.admin, private_room_id, tester.user1)
    print(f"Added user {tester.user1} to private room")
    
    # Test 5: Send message to private room
    print("\n5. Sending message to private room...")
    tester.send_message(tester.user1, private_room_id, "Hello,_private_room!")
    print("Message sent successfully")
    
    # Test 6: Try unauthorized access (should fail)
    print("\n6. Testing unauthorized access...")
    try:
        tester.send_message(tester.user2, private_room_id, "Unauthorized_message")
        print("ERROR: Unauthorized message succeeded when it should have failed")
    except Exception as e:
        if "moveabort" in str(e) and "abort_code" in str(e):
            print("Successfully caught unauthorized access attempt")
        else:
            raise e

if __name__ == "__main__":
    main()