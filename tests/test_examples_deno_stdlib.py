from pathlib import Path

import pytest

from tests.conftest import FlowHeaterLayer
from tests.util import execute

basedir = Path("examples/05-deno-stdlib")


@pytest.mark.xfail
def test_http_fetch(fh_http: FlowHeaterLayer):

    response = execute(basedir / "http-fetch.js")

    assert response.status_code == 200
    data = response.json()

    # Check STDOUT
    stdout = fh_http.get_stdout()
    print(stdout)

    assert "Example Domain" in stdout
