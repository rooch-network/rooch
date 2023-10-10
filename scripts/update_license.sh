#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

# Function to add license in a file
add_license() {
    local file="$1"
    local license="$2"

    # Check if the license already exists in the file and the "Copyright (c)" line contains "RoochNetwork"
    if ! grep -q 'SPDX-License-Identifier' "$file" || ! grep -q 'Copyright (c) RoochNetwork' "$file"; then
        # Add the license at the beginning of the file
        ex -s "$file" <<EOF
0i
${license}

.
wq
EOF
    fi
}

# Function to process .rs files
process_rs_files() {
    local directory="$1"

    # Find .rs files in the directory and its subdirectories
    find "$directory" -type f -name '*.rs' | while read -r file; do
        # Add license in the file
        add_license "$file" "// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0"
    done
}

# Function to process .move files
process_move_files() {
    local directory="$1"

    # Find .move files in the directory and its subdirectories
    find "$directory" -type f -name '*.move' | while read -r file; do
        # Add license in the file
        add_license "$file" "// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0"
    done
}

# Function to process .sh files
process_sh_files() {
    local directory="$1"

    find "$directory" -type f -name '*.sh' | while read -r file; do
        # Check if the license already exists at lines 2 and 3 if there's #!/bin/{bash or sh} at line 1
        if [[ $(head -n 1 "$file") =~ ^#!/bin/(bash|sh)$ ]]; then
            if ! head -n 3 "$file" | grep -q -e '# Copyright (c) RoochNetwork' -e '# SPDX-License-Identifier: Apache-2.0'; then
                ex -s "$file" <<EOF
1a
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

.
wq
EOF
            fi
        else
            # Add license below if there's no #!/bin/bash or #!/bin/sh at line 1
            ex -s "$file" <<EOF
0a
#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

.
wq
EOF
        fi
    done
}

# Start processing from the parent directory (../)
parent_directory="../"

# Process .rs files
process_rs_files "$parent_directory"

# Process .move files
process_move_files "$parent_directory"

# Process .sh files
process_sh_files "$parent_directory"
