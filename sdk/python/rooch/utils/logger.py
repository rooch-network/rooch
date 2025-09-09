#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import logging
from typing import Optional

# Configure root logger
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)

def get_logger(name: str, level: Optional[int] = None) -> logging.Logger:
    """Get a logger with the given name
    
    Args:
        name: Logger name (usually the module name)
        level: Optional logging level (defaults to INFO)
        
    Returns:
        A configured logger
    """
    logger = logging.getLogger(f"rooch:{name}")
    if level is not None:
        logger.setLevel(level)
    return logger