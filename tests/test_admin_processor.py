from dataclasses import dataclass, asdict
from typing import Optional

import requests
from requests import Response
from lovely.testlayers.server import ServerLayer


@dataclass
class RequestProcessor:
    id: Optional[str]
    name: str
    runtime: str
    language: str
    code: str


def create_request_processor(rp: RequestProcessor) -> Response:
    response = requests.post("http://localhost:3030/admin/processor", json=asdict(rp))
    data = response.json()

    assert len(data["id"]) > 0
    assert data["name"] == rp.name
    assert data["runtime"] == rp.runtime
    assert data["language"] == rp.language
    assert data["code"] == rp.code

    return response


def test_create_admin_processor(fh_http: ServerLayer):
    rp = RequestProcessor(id=None, name="testing", runtime="v8", language="js", code="my fun code")
    rp2 = RequestProcessor(id=None, name="testing2", runtime="v8", language="js", code="my fun code2")

    create_request_processor(rp)
    create_request_processor(rp2)


def test_get_admin_processor(fh_http: ServerLayer):
    rp = RequestProcessor(id=None, name="testing-get", runtime="v8", language="js", code="my fun code")
    response = create_request_processor(rp)
    data = response.json()

    response_get = requests.get(f"http://localhost:3030/admin/processor/{data['id']}")
    data_get = response_get.json()

    assert data["id"] == data_get["id"]
    assert data["name"] == data_get["name"]
    assert data["runtime"] == data_get["runtime"]
    assert data["language"] == data_get["language"]
    assert data["code"] == data_get["code"]


def test_update_admin_processor(fh_http: ServerLayer):
    rp = RequestProcessor(id=None, name="testing", runtime="v8", language="js", code="my fun code")
    response = create_request_processor(rp)
    data = response.json()

    rp.name = "testing-update"
    rp.code = "updated code"

    response_update = requests.put(f"http://localhost:3030/admin/processor/{data['id']}", json=asdict(rp))
    data_update = response_update.json()

    assert data["id"] == data_update["id"]
    assert "testing-update" == data_update["name"]
    assert data["runtime"] == data_update["runtime"]
    assert data["language"] == data_update["language"]
    assert "updated code" == data_update["code"]

    response_get = requests.get(f"http://localhost:3030/admin/processor/{data['id']}")
    data_get = response_get.json()

    assert data_update["id"] == data_get["id"]
    assert data_update["name"] == data_get["name"]
    assert data_update["runtime"] == data_get["runtime"]
    assert data_update["language"] == data_get["language"]
    assert data_update["code"] == data_get["code"]


def test_delete_admin_processor(fh_http: ServerLayer):
    rp = RequestProcessor(id=None, name="testing", runtime="v8", language="js", code="my fun code")

    response = create_request_processor(rp)
    rp_id = response.json()["id"]

    response_delete = requests.delete(f"http://localhost:3030/admin/processor/{rp_id}")
    assert 200 == response_delete.status_code

    # TODO: error cases do not work yet ;)
    # response_get = requests.get(f"http://localhost:3030/admin/processor/{rp_id}")
    # assert 404 == response_get.status_code
