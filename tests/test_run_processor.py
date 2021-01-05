from dataclasses import dataclass, asdict
from typing import Optional
from enum import Enum

import requests
from requests import Response
from lovely.testlayers.server import ServerLayer

from tests.test_admin_processor import RequestProcessor, create_request_processor


def test_run_processor(fh_http: ServerLayer):
    code = """
    // invoke ops
    Deno.core.ops();
    Deno.core.print(`Hello from DENO\n`);
    """

    rp = RequestProcessor(
        id=None,
        name="testing",
        runtime="v8",
        language="javascript",
        code=code,
    )

    response = create_request_processor(rp)
    rp_id = response.json()["id"]

    response_run = requests.post(f"http://localhost:3030/processor/{rp_id}/run")
    data = response_run.json()

    assert 200 == response_run.status_code

    # Check STDOUT
    fh_http.stdout.seek(0)
    stdout = fh_http.stdout.read()
    assert "Hello from DENO" in stdout
    assert "RUST: modified request is" in stdout


def test_json_availablity(fh_http: ServerLayer):
    code = """
    const data = {"a": "b"};
    const stringified = JSON.stringify(data);
    const matches = (obj, source) =>
        Object.keys(source).every(key => obj.hasOwnProperty(key) && obj[key] === source[key]);
    
    Deno.core.print(`Stringify: ${stringified}\n`);

    if (matches(JSON.parse(stringified), data)) {
        Deno.core.print(`Parse works, too\n`);
    } else {
        Deno.core.print(`PvD: ${JSON.parse(stringified)} v ${data}\n`);
    }
    """

    rp = RequestProcessor(
        id=None,
        name="testing",
        runtime="v8",
        language="javascript",
        code=code,
    )

    response = create_request_processor(rp)
    rp_id = response.json()["id"]

    response_run = requests.post(f"http://localhost:3030/processor/{rp_id}/run")
    data = response_run.json()
    print(data)
    assert 200 == response_run.status_code

    # Check STDOUT
    fh_http.stdout.seek(0)
    stdout = fh_http.stdout.read()
    print(stdout)
    assert 'Stringify: {"a":"b"}' in stdout
    assert "Parse works, too" in stdout
