#!/usr/bin/env python3
# Copyright (c) Rooch Network
# SPDX-License-Identifier: Apache-2.0

import os
from setuptools import setup, find_packages

# Read the README
with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

# Read version from version.py
version = {}
with open("rooch/version.py", "r") as fh:
    exec(fh.read(), version)

# Define package requirements
REQUIRES = [
    "aiohttp>=3.8.0",
    "pycryptodome>=3.18.0",
    "base58>=2.1.0",
    "typing_extensions>=4.5.0"
]

EXTRAS_REQUIRES = {
    "dev": [
        "pytest>=7.0.0",
        "pytest-asyncio>=0.20.0",
        "black>=23.0.0",
        "isort>=5.12.0",
        "mypy>=1.0.0",
        "flake8>=6.0.0",
    ]
}

setup(
    name="rooch-sdk",
    version=version["VERSION"],
    description="The official Rooch Python SDK",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="Rooch Network",
    author_email="developer@rooch.network",
    url="https://github.com/rooch-network/rooch",
    license="Apache-2.0",
    keywords="rooch, blockchain, move, python, sdk",
    packages=find_packages(exclude=["tests", "tests.*", "examples", "examples.*"]),
    python_requires=">=3.8",
    install_requires=REQUIRES,
    extras_require=EXTRAS_REQUIRES,
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: Apache Software License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Topic :: Software Development :: Libraries :: Python Modules",
    ],
    project_urls={
        "Documentation": "https://rooch.network/docs",
        "Source": "https://github.com/rooch-network/rooch/tree/main/sdk/python",
        "Bug Tracker": "https://github.com/rooch-network/rooch/issues",
    },
)