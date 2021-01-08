import requests

from tests.test_admin_processor import RequestProcessor, create_request_processor


def read_code(filename):
    with open(filename, "r") as f:
        return f.read()


def create_processor(filename):

    code = read_code(filename)

    rp = RequestProcessor(
        id=None,
        name="testing",
        runtime="v8",
        language="javascript",
        code=code,
    )

    response = create_request_processor(rp)
    rp_id = response.json()["id"]

    return rp_id


def run_processor(identifier, method="get", **kwargs):
    response = requests.request(
        method, f"http://localhost:3030/processor/{identifier}/run", **kwargs
    )

    if response.status_code < 300:
        assert "fh-conversation-id" in response.headers

    return response


def execute(filename, method="get", **kwargs):
    identifier = create_processor(filename)
    response = run_processor(identifier, method=method, **kwargs)
    return response
