from pathlib import Path

from dateutil.parser import parse

from tests.util import ApiClient

basedir = Path("examples/05-conversation")


def test_get_conversation(api_client: ApiClient):
    response = api_client.execute(basedir / "audit-item-logging.js", prelude=True)

    assert response.status_code == 200
    conversation_id = response.headers["fh-conversation-id"]

    (conversation, _response_conv) = api_client.get_request_conversation(
        conversation_id
    )
    assert conversation_id == conversation.id

    # check, if datetime parsing works
    parse(conversation.created_at)


def test_audit_item_logging(api_client: ApiClient):
    response = api_client.execute(basedir / "audit-item-logging.js", prelude=True)

    assert response.status_code == 200
    conversation_id = response.headers["fh-conversation-id"]

    (conversation, _response_conv) = api_client.get_request_conversation(
        conversation_id
    )
    assert 3 == len(conversation.audit_items)
    assert '"Hello, World"' == conversation.audit_items[1].payload
    assert '"Body is: "' == conversation.audit_items[2].payload


def test_audit_item_request(api_client: ApiClient):
    response = api_client.execute(basedir / "audit-item-request.js", prelude=True)

    assert response.status_code == 200
    conversation_id = response.headers["fh-conversation-id"]

    (conversation, _response_conv) = api_client.get_request_conversation(
        conversation_id
    )
    assert 3 == len(conversation.audit_items)
    assert "request" == conversation.audit_items[0].kind
    assert "" == conversation.audit_items[0].payload["body"]
    assert "GET" == conversation.audit_items[0].payload["method"]
    assert None == conversation.audit_items[0].payload["query"]

    assert "request" == conversation.audit_items[1].kind
    assert 0 == len(conversation.audit_items[1].payload["body"])

    assert "response" == conversation.audit_items[2].kind
    assert 0 == len(conversation.audit_items[1].payload["body"])
