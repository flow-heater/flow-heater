from dataclasses import asdict

import pytest
from lovely.testlayers.server import ServerLayer

from tests.util import ApiClient, RequestProcessor


@pytest.mark.admin
def test_create_admin_processor(api_client: ApiClient):
    rp = RequestProcessor(
        id=None, name="testing", runtime="v8", language="javascript", code="my fun code"
    )
    rp2 = RequestProcessor(
        id=None,
        name="testing2",
        runtime="v8",
        language="javascript",
        code="my fun code2",
    )

    api_client.create_request_processor(rp)
    api_client.create_request_processor(rp2)


@pytest.mark.admin
def test_get_admin_processor(api_client: ApiClient):
    rp = RequestProcessor(
        id=None,
        name="testing-get",
        runtime="v8",
        language="javascript",
        code="my fun code",
    )
    response = api_client.create_request_processor(rp)
    data = response.json()

    response_get = api_client.http_client.get(f"/admin/processor/{data['id']}")
    data_get = response_get.json()

    assert data["id"] == data_get["id"]
    assert data["name"] == data_get["name"]
    assert data["runtime"] == data_get["runtime"]
    assert data["language"] == data_get["language"]
    assert data["code"] == data_get["code"]


@pytest.mark.admin
def test_update_admin_processor(api_client: ApiClient):
    rp = RequestProcessor(
        id=None, name="testing", runtime="v8", language="javascript", code="my fun code"
    )
    response = api_client.create_request_processor(rp)
    data = response.json()

    rp.name = "testing-update"
    rp.code = "updated code"

    response_update = api_client.http_client.put(
        f"/admin/processor/{data['id']}", json=asdict(rp)
    )
    data_update = response_update.json()

    assert data["id"] == data_update["id"]
    assert "testing-update" == data_update["name"]
    assert data["runtime"] == data_update["runtime"]
    assert data["language"] == data_update["language"]
    assert "updated code" == data_update["code"]

    response_get = api_client.http_client.get(f"/admin/processor/{data['id']}")
    data_get = response_get.json()

    assert data_update["id"] == data_get["id"]
    assert data_update["name"] == data_get["name"]
    assert data_update["runtime"] == data_get["runtime"]
    assert data_update["language"] == data_get["language"]
    assert data_update["code"] == data_get["code"]


@pytest.mark.admin
def test_delete_admin_processor(api_client: ApiClient):
    rp = RequestProcessor(
        id=None,
        name="testing",
        runtime="v8",
        language="javascript",
        code="my fun code",
    )

    response = api_client.create_request_processor(rp)
    rp_id = response.json()["id"]
    response_delete = api_client.http_client.delete(f"/admin/processor/{rp_id}")
    assert 200 == response_delete.status_code

    # check if it's really gone
    response_get = api_client.http_client.get(f"/admin/processor/{rp_id}")
    assert 404 == response_get.status_code


@pytest.mark.admin
def test_not_found_processor(api_client: ApiClient):
    id = "8a2e00e9-c710-4337-b717-bdcad0396df5"
    assert 404 == api_client.http_client.post(f"/processor/{id}/run").status_code
    assert 404 == api_client.http_client.get(f"/admin/processor/{id}").status_code
    resp = api_client.http_client.delete(f"/admin/processor/{id}")
    data = resp.json()

    assert 404 == resp.status_code
    assert data["code"] == 404
    assert f"with id {id} not found" in data["message"]
