import tempfile

import pytest
from fastapi.testclient import TestClient
from fh.gateway.app import app as FastApiApp
from fh.gateway.config import Config
from lovely.testlayers.server import ServerLayer

from tests.util import ApiClient


class FlowHeaterLayer(ServerLayer):
    """
    Basic ServerLayer which runs the rust core binary.
    """

    def __init__(self, config: Config):
        stdout = tempfile.NamedTemporaryFile(mode="w+")
        stderr = tempfile.NamedTemporaryFile(mode="w+")
        super(FlowHeaterLayer, self).__init__(
            name="fh-core",
            servers=[f"{config.core.host}:{config.core.port}"],
            start_cmd="cargo run --bin fh-http",
            stdout=stdout,
            stderr=stderr,
        )


@pytest.fixture(scope="session")
def config():
    """
    Just loads a config from the prepared environment.
    """
    return Config.from_env()


@pytest.fixture(scope="session")
def fh_core(config: Config) -> FlowHeaterLayer:
    """
    Spins up the rust core binary.
    """
    server = FlowHeaterLayer(config)
    server.setUp()
    yield server
    server.tearDown()


@pytest.fixture(scope="session")
def api_client(config: Config, fh_core: FlowHeaterLayer) -> ApiClient:
    """
    Creates an `ApiClient`, which wraps a FastApi TestClient.
    """
    client = TestClient(FastApiApp)
    gwc = ApiClient(client)
    yield gwc
