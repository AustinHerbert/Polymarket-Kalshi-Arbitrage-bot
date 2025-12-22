#!/usr/bin/env python3
"""Test Kalshi signature - run this and compare to Rust output"""
import time
import base64
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding

KEY_PATH = "/Users/austinherbert/Documents/kalshi_private_key.pem"

# Load key
with open(KEY_PATH, 'rb') as f:
    private_key = serialization.load_pem_private_key(f.read(), password=None)

# Use a fixed timestamp for comparison
timestamp = "1735000000000"
message = f"{timestamp}GET/trade-api/ws/v2"

print(f"Message: '{message}'")
print(f"Key size (bits): {private_key.key_size}")
print(f"Key size (bytes): {private_key.key_size // 8}")

# Calculate max salt length
key_bytes = private_key.key_size // 8
hash_len = 32  # SHA256
max_salt = key_bytes - hash_len - 2
print(f"Max salt length: {max_salt}")

# Sign with MAX_LENGTH
signature = private_key.sign(
    message.encode(),
    padding.PSS(
        mgf=padding.MGF1(hashes.SHA256()),
        salt_length=padding.PSS.MAX_LENGTH
    ),
    hashes.SHA256()
)
sig_b64 = base64.b64encode(signature).decode()

print(f"Signature length: {len(sig_b64)}")
print(f"Signature (first 50): {sig_b64[:50]}")
