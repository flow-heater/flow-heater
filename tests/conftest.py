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
def test_user_jwt():
    """"Dummy JWT."""
    return "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IkJSQ2xFQkFVMXNVRVZIWU44dk5PYSJ9.eyJuaWNrbmFtZSI6InRpbSIsIm5hbWUiOiJ0aW1AZWxiYXJ0LmNvbSIsInBpY3R1cmUiOiJodHRwczovL3MuZ3JhdmF0YXIuY29tL2F2YXRhci85NGI0M2NmZGNiNDg3NTFmMTIwODc0MWRiNzdmZWZhOT9zPTQ4MCZyPXBnJmQ9aHR0cHMlM0ElMkYlMkZjZG4uYXV0aDAuY29tJTJGYXZhdGFycyUyRnRpLnBuZyIsInVwZGF0ZWRfYXQiOiIyMDIxLTAyLTIxVDIxOjE0OjQyLjE5OVoiLCJlbWFpbCI6InRpbUBlbGJhcnQuY29tIiwiZW1haWxfdmVyaWZpZWQiOnRydWUsImlzcyI6Imh0dHBzOi8vZmxvdy1oZWF0ZXIuZXUuYXV0aDAuY29tLyIsInN1YiI6ImF1dGgwfDYwMjNlZTZkNzI2NjY1MDA2YTI5MDNmYSIsImF1ZCI6IlZxNWhDM0ZlWlNxOWdqYVBJNWhkU2hNelQ1dnc4ODZvIiwiaWF0IjoxNjEzOTQyMDgyLCJleHAiOjE2MTM5NzgwODIsIm5vbmNlIjoiTGhNWHdoekJLNTlZclRJVnVZUnUifQ.WtYMf2KabF3D9GUEMSUgNoB3mYkc1jlBgMgQm7D5t6t9pmE0_2c1VQ6afilJznxPHgQMQCxCYweznDAKF8a9nhmn6Gwt_bTyNWjSYFT2-lcUXL-JAIl-qJx37NwRwgvmYRZZXGoS-TfiuMVOob4r-H7nkzbgM-JjSt8kuejzfWkhZ_SX4H71zpr6KMR219JK6qLzzNoAPGfaDbS_9xBjkshtRUwRVTWlX8WqLfSLLI6ukW7v1_cJbcA3-0OwFsdY2px1zFoanonxqCJN63FZA1nsu3iS2z5zYjKov1g-tVDSLYVEIJRsgIyIB7sxK29sFOV6r7UaxkhR7xrPFi7YMA"


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
def api_client(config: Config, fh_core: FlowHeaterLayer, test_user_jwt) -> ApiClient:
    """
    Creates an `ApiClient`, which wraps a FastApi TestClient.
    """
    client = TestClient(FastApiApp)
    client.headers["authorization"] = f"Bearer {test_user_jwt}"
    gwc = ApiClient(client)
    yield gwc
