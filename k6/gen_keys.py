from base64 import b64encode

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from cryptography.hazmat.primitives.asymmetric.x25519 import X25519PrivateKey

ed25519 = Ed25519PrivateKey.generate()
x25519 = X25519PrivateKey.generate()

signature = ed25519.sign(x25519.public_key().public_bytes_raw() + ed25519.public_key().public_bytes_raw())

print(f"x25519: \"{str(b64encode(x25519.public_key().public_bytes_raw()).strip(b'='), encoding='ascii')}\",")
print(f"ed25519: \"{str(b64encode(ed25519.public_key().public_bytes_raw()).strip(b'='), encoding='ascii')}\",")
print(f"signature: \"{str(b64encode(signature).strip(b'='), encoding='ascii')}\",")

print()

new_ed25519 = Ed25519PrivateKey.generate()
new_x25519 = X25519PrivateKey.generate()
new_signature = new_ed25519.sign(new_x25519.public_key().public_bytes_raw() + new_ed25519.public_key().public_bytes_raw())
authorization = ed25519.sign(new_signature)

print(f"x25519: \"{str(b64encode(new_x25519.public_key().public_bytes_raw()).strip(b'='), encoding='ascii')}\",")
print(f"ed25519: \"{str(b64encode(new_ed25519.public_key().public_bytes_raw()).strip(b'='), encoding='ascii')}\",")
print(f"signature: \"{str(b64encode(new_signature).strip(b'='), encoding='ascii')}\",")
print(f"authorization: \"{str(b64encode(authorization).strip(b'='), encoding='ascii')}\",")
