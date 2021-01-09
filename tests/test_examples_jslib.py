import json
from pathlib import Path

from tests.conftest import FlowHeaterLayer
from tests.util import execute
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

    # Check STDOUT
    stdout = fh_http.get_stdout()
    assert stdout == "Hello world."


def test_modcaesar(fh_http: FlowHeaterLayer):

    # ``fh-core`` doesn't know how to interpret
    # the non-standard ``@fh:include()`` yet.
    # So, let's resolve this in user space.
    jsfile = basedir / "02-caesar-cipher/userspace.js"
    jscode = preprocess_javascript(jsfile)

    response = execute(jscode, data={"payload": "Hello world."})
    assert response.status_code == 200

    # Check STDOUT
    stdout = fh_http.get_stdout()
    assert "encoded: TQXXA IADXP." in stdout
    assert "decoded: HELLO WORLD." in stdout


def test_htmlparser(fh_http: FlowHeaterLayer):

    # ``fh-core`` doesn't know how to interpret
    # the non-standard ``@fh:include()`` yet.
    # So, let's resolve this in user space.
    jsfile = basedir / "05-htmlparser/parse-html.js"
    jscode = preprocess_javascript(jsfile)

    response = execute(jscode)

    assert response.status_code == 200
    data = response.json()

    # Check STDOUT
    stdout = fh_http.get_stdout()

    data = json.loads(stdout)

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
