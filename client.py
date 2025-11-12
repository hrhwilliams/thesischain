import requests as req


def register_user(port: int, user: str, key: str):
    """Attempt to publish a user and their public key"""
    response = req.post(f"http://localhost:{port}/api/register", json={
        "name": user,
        "key": key
    }, timeout=10)

    try:
        response.raise_for_status()
    except req.exceptions.HTTPError as _:
        try:
            print(response.json())
        except ValueError:
            print(response.text)
