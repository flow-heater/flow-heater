import requests
from lovely.testlayers.server import ServerLayer


def test_spike(fh_http: ServerLayer):

    response = requests.post("http://localhost:3030/hello/xxx", json={"a": "b"})
    data = response.json()

    # Check HTTP response.
    assert data["code"] == 204
    assert "fh-conversation-id" in response.headers
    assert data["body"] == "xxx"
    assert data["headers"]["content-type"][0] == "application/xml"

    # Check STDOUT for fun.
    fh_http.stdout.seek(0)
    stdout = fh_http.stdout.read()
    assert "DENO: Got request body" in stdout
