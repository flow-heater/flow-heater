import json
import requests

from lovely.testlayers.server import ServerLayer
from tests.util import get_conversation_from_response


def test_spike(fh_http: ServerLayer):

    response = requests.post("http://localhost:3030/hello/xxx", json={"a": "b"})
    data = response.json()

    # Check HTTP response.
    assert data["code"] == 204
    assert "fh-conversation-id" in response.headers
    assert data["body"] == "xxx"
    assert data["headers"]["content-type"][0] == "application/xml"

    # Fetch RequestConversation
    conversation = get_conversation_from_response(response)
    assert 5 == len(conversation.audit_items)

    # Check Log entries
    assert "log" == conversation.audit_items[1].kind
    assert "DENO: Got request body" in json.loads(conversation.audit_items[1].payload)
