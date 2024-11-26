// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

package types

// PaginationArguments represents parameters for pagination
type PaginationArguments struct {
    // Optional paging cursor, using interface{} to allow for different cursor types
    Cursor interface{} `json:"cursor,omitempty"`
    // Maximum item returned per page
    Limit *int `json:"limit,omitempty"`
}

// PaginationResult represents the paginated response
type PaginationResult struct {
    Cursor     interface{} `json:"cursor,omitempty"`
    Data       interface{} `json:"data"`
    HasNextPage bool       `json:"hasNextPage"`
}

// SessionInfoView represents session information
type SessionInfoView struct {
    AppName              string   `json:"appName"`
    AppURL               string   `json:"appUrl"`
    AuthenticationKey    string   `json:"authenticationKey"`
    Scopes              []string `json:"scopes"`
    CreateTime          int64    `json:"createTime"`
    LastActiveTime      int64    `json:"lastActiveTime"`
    MaxInactiveInterval int64    `json:"maxInactiveInterval"`
} 