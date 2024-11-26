package utils

import "fmt"

// validateWitness checks if the witness program is valid according to BIP-0141 rules
func ValidateWitness(version int, data []byte) error {
    if len(data) < 2 || len(data) > 40 {
        return fmt.Errorf("Witness: invalid length")
    }
    
    if version > 16 {
        return fmt.Errorf("Witness: invalid version")
    }
    
    if version == 0 && !(len(data) == 20 || len(data) == 32) {
        return fmt.Errorf("Witness: invalid length for version")
    }
    
    return nil
} 