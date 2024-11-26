package client

// RoochTransport defines the interface for making RPC requests
type RoochTransport interface {
    Request(method string, params interface{}, result interface{}) error
}

// RoochHTTPTransport implements RoochTransport for HTTP
type RoochHTTPTransport struct {
    url string
    // Add HTTP client configuration here
}

func NewRoochHTTPTransport(url string) *RoochHTTPTransport {
    return &RoochHTTPTransport{
        url: url,
    }
}

func (t *RoochHTTPTransport) Request(method string, params interface{}, result interface{}) error {
    // Implement HTTP request logic here
    return nil
} 