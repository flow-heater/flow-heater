import json

from tests.util import ApiClient


def test_spike(
    api_client: ApiClient,
):
    response = api_client.http_client.post("/hello/xxx", json={"a": "b"})
    data = response.json()

    # Check HTTP response.
    assert data["code"] == 204
    assert "fh-conversation-id" in response.headers
    assert data["body"] == "xxx"
    assert data["headers"]["content-type"][0] == "application/xml"

    # Fetch RequestConversation
    conversation = api_client.get_conversation_from_response(response)
    assert 5 == len(conversation.audit_items)

    # Check Log entries
    assert "log" == conversation.audit_items[1].kind
    assert "DENO: Got request body" in json.loads(conversation.audit_items[1].payload)
