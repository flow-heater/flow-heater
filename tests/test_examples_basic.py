import json
from pathlib import Path

from tests.util import ApiClient

basedir = Path("examples/01-basic")


def test_error_syntax(api_client: ApiClient):
    """
    $ deno run examples/01-basic/error-syntax.js
    error: Expected ';', '}' or <eof> at file:///Users/amo/dev/flow-heater/fh-core/examples/01-basic/error-syntax.js:1:6

    As this is actually a linter error, respective code should not
    even make it to runtime when already caught on insertion time.
    See https://github.com/flow-heater/fh-core/issues/22.
    """

    response = api_client.execute(
        basedir / "error-syntax.js", headers={"FH-Debug": "true"}
    )

    assert response.status_code == 500
    data = response.json()

    # TODO: That might yield a linter error after the HTTP request
    #       header "FH-Debug: true" will get evaluated appropriately.


def test_error_runtime(api_client: ApiClient):
    """
    $ deno run examples/01-basic/error-runtime.js
    error: Uncaught TypeError: JSON.foobar is not a function
    JSON.foobar();
         ^
        at file:///Users/amo/dev/flow-heater/fh-core/examples/01-basic/error-runtime.js:1:6

    For propagating those runtime errors to the user interactively,
    I've elaborated on the `FH-Debug: true` header within
    https://github.com/flow-heater/fh-core/issues/33.
    """

    response = api_client.execute(
        basedir / "error-runtime.js", headers={"FH-Debug": "true"}
    )

    assert response.status_code == 500
    data = response.json()

    # TODO: That might yield a stacktrace after the HTTP request
    #       header "FH-Debug: true" will get evaluated appropriately.


def test_json_demo(api_client: ApiClient):

    response = api_client.execute(basedir / "json-demo.js")

    assert response.status_code == 200

    # Fetch RequestConversation
    conversation = api_client.get_conversation_from_response(response)
    assert 3 == len(conversation.audit_items)

    # Check Log entries
    assert "log" == conversation.audit_items[1].kind
    assert 'Stringify: {"a":"b"}' in json.loads(conversation.audit_items[1].payload)

    assert conversation.audit_items[2].kind == "log"
    assert "Parse works, too" in json.loads(conversation.audit_items[2].payload)


def test_json_request_get(api_client: ApiClient):

    response = api_client.execute(
        basedir / "json-request-echo.js", headers={"foo": "bar"}
    )

    assert response.status_code == 200

    # Fetch RequestConversation
    conversation = api_client.get_conversation_from_response(response)
    assert 2 == len(conversation.audit_items)

    # Check Log entries
    assert "log" == conversation.audit_items[1].kind
    data = json.loads(json.loads(conversation.audit_items[1].payload))
    print(data)

    assert data["method"] == "GET"
    assert data["headers"]["user-agent"][0].startswith("testclient")
    assert data["headers"]["foo"][0] == "bar"
    assert data["body"] == ""


def test_json_request_post(api_client: ApiClient):

    response = api_client.execute(basedir / "json-request-echo.js", method="post")

    assert response.status_code == 200

    # Fetch RequestConversation
    conversation = api_client.get_conversation_from_response(response)
    assert 2 == len(conversation.audit_items)

    # Check Log entries
    assert "log" == conversation.audit_items[1].kind
    data = json.loads(json.loads(conversation.audit_items[1].payload))
    print(data)

    assert data["method"] == "POST"
    assert data["body"] == ""


def test_json_request_post_x_www_form_urlencoded(api_client: ApiClient):

    response = api_client.execute(
        basedir / "json-request-echo.js", method="post", data={"foo": "bar"}
    )

    assert response.status_code == 200

    # Fetch RequestConversation
    conversation = api_client.get_conversation_from_response(response)
    assert 2 == len(conversation.audit_items)

    # Check Log entries
    assert "log" == conversation.audit_items[1].kind
    data = json.loads(json.loads(conversation.audit_items[1].payload))
    print(data)
    assert data["method"] == "POST"
    assert data["body"] == "foo=bar"


def test_json_request_post_json(api_client: ApiClient):

    response = api_client.execute(
        basedir / "json-request-echo.js", method="post", json={"foo": "bar"}
    )

    assert response.status_code == 200

    # Fetch RequestConversation
    conversation = api_client.get_conversation_from_response(response)
    assert 2 == len(conversation.audit_items)

    # Check Log entries
    assert "log" == conversation.audit_items[1].kind
    data = json.loads(json.loads(conversation.audit_items[1].payload))
    print(data)
    assert data["method"] == "POST"
    assert data["body"] == '{"foo": "bar"}'
