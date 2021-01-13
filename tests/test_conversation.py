from pathlib import Path
from dateutil.parser import parse

from tests.conftest import FlowHeaterLayer
from tests.util import execute, get_request_conversation

basedir = Path("examples/05-conversation")


def test_get_conversation(fh_http: FlowHeaterLayer):
    response = execute(basedir / "audit-item-logging.js", prelude=True)

    assert response.status_code == 200
    conversation_id = response.headers["fh-conversation-id"]

    (conversation, _response_conv) = get_request_conversation(conversation_id)
    assert conversation_id == conversation.id

    # check, if datetime parsing works
    parse(conversation.created_at)


def test_audit_item_logging(fh_http: FlowHeaterLayer):
    response = execute(basedir / "audit-item-logging.js", prelude=True)

    assert response.status_code == 200
    conversation_id = response.headers["fh-conversation-id"]

    (conversation, _response_conv) = get_request_conversation(conversation_id)
    assert 2 == len(conversation.audit_items)
    assert '"Hello, World"' == conversation.audit_items[0].payload
    assert '"Body is: "' == conversation.audit_items[1].payload


def test_audit_item_request(fh_http: FlowHeaterLayer):
    response = execute(basedir / "audit-item-request.js", prelude=True)

    assert response.status_code == 200
    conversation_id = response.headers["fh-conversation-id"]

    (conversation, _response_conv) = get_request_conversation(conversation_id)
    assert 2 == len(conversation.audit_items)
    assert "request" == conversation.audit_items[0].kind
    assert "" == conversation.audit_items[0].payload["body"]
    assert "GET" == conversation.audit_items[0].payload["method"]
    assert None == conversation.audit_items[0].payload["query"]

    assert "response" == conversation.audit_items[1].kind
    assert 0 < len(conversation.audit_items[1].payload["body"])
