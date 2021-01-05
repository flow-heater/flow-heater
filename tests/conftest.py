import tempfile

import pytest
from lovely.testlayers.server import ServerLayer


class FlowHeaterLayer(ServerLayer):
    def __init__(self):
        stdout = tempfile.NamedTemporaryFile(mode="w+")
        stderr = tempfile.NamedTemporaryFile(mode="w+")
        super(FlowHeaterLayer, self).__init__(
            name="fh-http",
            servers=["localhost:3030"],
            start_cmd="cargo run --bin fh-http",
            stdout=stdout,
            stderr=stderr,
        )

    def get_stdout(self, strip_nulls=True):
        self.stdout.seek(0)
        payload = self.stdout.read()
        # FIXME: There are a bunch of null-bytes at the beginning of STDOUT. Why?
        if strip_nulls:
            payload = payload.strip("\x00")
        return payload


@pytest.fixture(scope="session")
def fh_http():
    server = FlowHeaterLayer()
    server.setUp()
    yield server
    server.tearDown()


@pytest.fixture(scope="function", autouse=True)
def fh_stdout(fh_http):
    fh_http.stdout.truncate(0)
