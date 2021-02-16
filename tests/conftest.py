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


class FHGatewayLayer(ServerLayer):
    def __init__(self):
        stdout = tempfile.NamedTemporaryFile(mode="w+")
        stderr = tempfile.NamedTemporaryFile(mode="w+")
        super(FHGatewayLayer, self).__init__(
            name="fh-gateway",
            servers=["localhost:3031"],
            # when using `just run-gateway` here, the tearDown() fails (waits forever)
            start_cmd=".venv/bin/uvicorn fh.gateway.app:app --app-dir fh-gateway --port 3031",
            stdout=stdout,
            stderr=stderr,
        )


@pytest.fixture(scope="session")
def fh_http():
    server = FlowHeaterLayer()
    server.setUp()
    yield server
    server.tearDown()


@pytest.fixture(scope="session")
def fh_gateway():
    server = FHGatewayLayer()
    server.setUp()
    yield server
    server.tearDown()
