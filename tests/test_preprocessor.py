import sys
from pathlib import Path
from unittest import mock

import pytest
import responses

from tests.preprocessor import get_path_safe, load_resource, preprocess_javascript


@mock.patch("tests.preprocessor.load_resource", return_value="bazqux")
def test_preprocess_include_filesystem(load_resource_patched: mock.Mock):
    jscode_before = '// @fh:include("./foobar.js")'
    jscode_after = preprocess_javascript(code=jscode_before)
    load_resource_patched.assert_called_once()
    assert jscode_after == "bazqux"


@responses.activate
def test_preprocess_include_url():
    responses.add(responses.GET, "http://example.org/foobar.js", body="bazqux")
    jscode_before = '// @fh:include("http://example.org/foobar.js")'
    jscode_after = preprocess_javascript(code=jscode_before)
    assert responses.assert_call_count("http://example.org/foobar.js", 1)
    assert jscode_after == "bazqux"


def test_load_resource_filesystem():
    """
    Load this testfile to demonstrate filesystem loading works.
    """
    basedir = Path(__file__).parent
    resource = Path(__file__).name
    payload = load_resource(basedir=basedir, resource=resource)
    assert "test_load_resource_filesystem" in payload


@responses.activate
def test_load_resource_url():
    responses.add(responses.GET, "http://example.org/foobar.js", body="bazqux")
    payload = load_resource(basedir=None, resource="http://example.org/foobar.js")
    assert payload == "bazqux"


def test_get_path_safe_success():
    outcome = get_path_safe(
        basedir=Path("./path/to"), resource="./relative/resource.js"
    )
    assert outcome == Path("path/to/relative/resource.js")


def test_get_path_safe_fail_traversal():
    with pytest.raises(ValueError) as ex:
        get_path_safe(basedir=Path("./path/to"), resource="../../../something/fishy.js")

    if sys.version_info >= (3, 9):
        assert ex.match(
            "'.*?/something/fishy.js' is not in the subpath of '.*?' "
            "OR one path is relative and the other is absolute."
        )
    else:
        assert ex.match("'.*?/something/fishy.js' does not start with '.*?'")
