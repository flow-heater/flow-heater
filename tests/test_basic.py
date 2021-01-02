import requests
from lovely.testlayers.server import ServerLayer


def test_spike(fh_http: ServerLayer):

    response = requests.post("http://localhost:3030/hello/xxx", json={"a": "b"})
    data = response.json()

    # Check HTTP response.
    assert data["code"] == 200
    assert data["headers"] == {}
    assert data["body"] == [
        116,
        104,
        105,
        115,
        32,
        105,
        115,
        32,
        116,
        104,
        101,
        32,
        112,
        97,
        116,
        99,
        104,
        101,
        100,
        32,
        98,
        111,
        100,
        121,
    ]

    # Check STDOUT for fun.
    fh_http.stdout.seek(0)
    stdout = fh_http.stdout.read()
    assert "DENO: Got request body" in stdout
    assert "RUST: modified request is" in stdout
