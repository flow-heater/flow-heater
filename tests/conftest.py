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


@pytest.fixture(scope="session")
def fh_http():
    server = FlowHeaterLayer()
    server.setUp()
    yield server
    server.tearDown()
