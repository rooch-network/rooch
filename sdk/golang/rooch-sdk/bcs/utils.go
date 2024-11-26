package bcs

import (
	"strings"
)

func SplitGenericParameters(str string) []string {
	var result []string
	var current string
	var depth int

	for _, char := range str {
		switch char {
		case '<':
			depth++
			current += string(char)
		case '>':
			depth--
			current += string(char)
		case ',':
			if depth == 0 {
				result = append(result, strings.TrimSpace(current))
				current = ""
			} else {
				current += string(char)
			}
		default:
			current += string(char)
		}
	}

	if current != "" {
		result = append(result, strings.TrimSpace(current))
	}

	return result
} 