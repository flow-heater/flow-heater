import json
from pathlib import Path

from tests.conftest import FlowHeaterLayer
from tests.util import execute, get_conversation_from_response
from tests.preprocessor import preprocess_javascript

basedir = Path("examples/06-javascript-libs")


def test_modhello(fh_http: FlowHeaterLayer):

    # ``fh-core`` doesn't know how to interpret
    # the non-standard ``@fh:include()`` yet.
    # So, let's resolve this in user space.
    jsfile = basedir / "01-basic-module/userspace.js"
    jscode = preprocess_javascript(jsfile)

    response = execute(jscode)
    assert response.status_code == 200

    # Fetch RequestConversation
    conversation = get_conversation_from_response(response)
    assert 2 == len(conversation.audit_items)

    # Check Log entries
    assert "log" == conversation.audit_items[1].kind
    assert "Hello world." == json.loads(conversation.audit_items[1].payload)


def test_modcaesar(fh_http: FlowHeaterLayer):

    # ``fh-core`` doesn't know how to interpret
    # the non-standard ``@fh:include()`` yet.
    # So, let's resolve this in user space.
    jsfile = basedir / "02-caesar-cipher/userspace.js"
    jscode = preprocess_javascript(jsfile)

    response = execute(jscode, data={"payload": "Hello world."})
    assert response.status_code == 200

    # Fetch RequestConversation
    conversation = get_conversation_from_response(response)
    assert 3 == len(conversation.audit_items)

    # Check Log entries
    assert "log" == conversation.audit_items[1].kind
    assert "encoded: TQXXA IADXP.\n" == json.loads(conversation.audit_items[1].payload)

    assert "log" == conversation.audit_items[2].kind
    assert "decoded: HELLO WORLD.\n" == json.loads(conversation.audit_items[2].payload)


def test_htmlparser(fh_http: FlowHeaterLayer):

    # ``fh-core`` doesn't know how to interpret
    # the non-standard ``@fh:include()`` yet.
    # So, let's resolve this in user space.
    jsfile = basedir / "05-htmlparser/parse-html.js"
    jscode = preprocess_javascript(jsfile)

    response = execute(jscode)

    assert response.status_code == 200

    # Fetch RequestConversation
    conversation = get_conversation_from_response(response)
    assert 2 == len(conversation.audit_items)

    # Check log entries
    assert "log" == conversation.audit_items[1].kind
    data = json.loads(json.loads(conversation.audit_items[1].payload))
    print(data)

    assert data == {
        "type": 9,
        "content": [
            "",
            {
                "type": 1,
                "content": [
                    "",
                    {
                        "type": 1,
                        "content": ["David Bowie"],
                        "name": "name",
                        "attributes": {},
                    },
                    "",
                ],
                "name": "document",
                "attributes": {"attribute": "value"},
            },
        ],
    }
