#!/usr/bin/env python3

from rooch.crypto.keypair import KeyPair
from rooch.crypto.signer import RoochSigner
from rooch.transactions.builder import TransactionBuilder

def test_address_consistency():
    """Test that KeyPair.get_rooch_address() matches TransactionBuilder's bitcoin address derivation"""
    test_kp = KeyPair.from_seed("test_seed_for_integration")
    signer = RoochSigner(test_kp)
    
    # Get address from KeyPair
    keypair_address = str(test_kp.get_rooch_address())
    
    # Get address from signer
    signer_address = signer.get_address()
    
    print(f"\nAddress comparison:")
    print(f"KeyPair.get_rooch_address():  {keypair_address}")
    print(f"RoochSigner.get_address():    {signer_address}")
    print(f"Addresses match:              {keypair_address == signer_address}")
    
    # Also test with TransactionBuilder
    builder = TransactionBuilder(
        sender_address=signer_address,
        sequence_number=0,
        chain_id=4,
        max_gas_amount=10_000_000
    )
    
    bitcoin_rooch_address = builder._get_bitcoin_rooch_address(signer)
    print(f"TransactionBuilder Bitcoin addr: {bitcoin_rooch_address}")
    print(f"All addresses match:             {keypair_address == signer_address == bitcoin_rooch_address}")
    
    if keypair_address == signer_address == bitcoin_rooch_address:
        print("✅ All address generation methods are consistent!")
        
        # Now we can remove the auto-correction logic since KeyPair.get_rooch_address() is fixed
        print("\n🔧 Since KeyPair.get_rooch_address() is now correct, we can remove the temporary auto-correction mechanism.")
        return True
    else:
        print("❌ Address generation methods are still inconsistent!")
        return False

if __name__ == "__main__":
    test_address_consistency()
