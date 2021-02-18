from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Union

import requests
from dacite import from_dict
from fastapi.testclient import TestClient


@dataclass
class RequestProcessor:
    """
    Represents a JSON API object for a Request Processor.
    """

    id: Optional[str]
    name: str
    runtime: str
    language: str
    code: str


@dataclass
class AuditItem:
    """
    Represents a JSON API object for an Audit Item.
    """

    id: str
    kind: str
    created_at: str
    conversation_id: str
    payload: Union[str, Dict]
    inc: Optional[int]
    request_id: Optional[str]


@dataclass
class RequestConversation:
    """
    Represents a JSON API object for a Request Conversation.
    """

    id: str
    request_processor_id: str
    created_at: str
    audit_items: List[AuditItem]


def read_code(filename_or_code: Union[Path, str]) -> str:
    """
    Reads a code file from the filesystem if the given variable is a `Path`.
    Otherwise simply returns the given `str`.
    """
    if isinstance(filename_or_code, Path):
        with open(filename_or_code, "r") as f:
            return f.read()
    else:
        return filename_or_code


def wrap_with_async_main(code: str) -> str:
    """
    Wraps the given code string with an JavaScript `async function main()`.
    Convenience method which is used very often. In case the api for the main()
    function changes, it's easily adaptable.
    """
    return f"""
    async function main(fh, request) {{
        {code}
    }}
    """


class ApiClient:
    """
    Wraps api functionalities:
    - the raw `TestClient` to make "raw" requests to the python application
    - higher level abstractions for getting Request Conversations, creating
      Request Processors, ...
    """

    def __init__(self, http_client: TestClient):
        self.http_client = http_client

    def create_processor(self, code: str):
        """
        Creates a Request Processor with the given code string. Convenience
        wrapper for the `create_request_processor()` method.
        """
        rp = RequestProcessor(
            id=None,
            name="testing",
            runtime="v8",
            language="javascript",
            code=code,
        )

        response = self.create_request_processor(rp)
        rp_id = response.json()["id"]

        return rp_id

    def create_request_processor(self, rp: RequestProcessor) -> requests.Response:
        """
        Creates a Request Processor with the given `RequestProcessor`object.
        """
        response = self.http_client.post("/admin/processor", json=asdict(rp))
        data = response.json()

        assert len(data["id"]) > 0
        assert data["name"] == rp.name
        assert data["runtime"] == rp.runtime
        assert data["language"] == rp.language
        assert data["code"] == rp.code

        return response

    def run_processor(
        self, identifier, method="get", prelude=True, **kwargs
    ) -> requests.Response:
        """
        Runs a request processor.
        """
        path = "run" if not prelude else "run_with_prelude"
        response = self.http_client.request(
            method, f"/processor/{identifier}/{path}", **kwargs
        )

        if response.status_code < 300:
            assert "fh-conversation-id" in response.headers

        return response

    def execute(
        self, filename_or_code: Union[Path, str], method="get", prelude=True, **kwargs
    ):
        """
        Convenience wrapper which takes the given path or code string and then:
        - creates the Request Processor,
        - runs the Request Processor.
        """
        code = read_code(filename_or_code)

        if prelude:
            code = wrap_with_async_main(code)

        identifier = self.create_processor(code)
        response = self.run_processor(
            identifier, method=method, prelude=prelude, **kwargs
        )

        return response

    def get_request_conversation(
        self, conversation_id: str
    ) -> Tuple[RequestConversation, requests.Response]:
        """
        Fetches a Request Conversation from the API.
        """
        response = self.http_client.get(f"/conversation/{conversation_id}")
        return (
            from_dict(data_class=RequestConversation, data=response.json()),
            response,
        )

    def get_conversation_from_response(
        self, response: requests.Response
    ) -> RequestConversation:
        """
        Extracts the Conversation Id from the given responses HTTP header and
        then fetches the Request Conversation from the API.
        """
        return self.get_request_conversation(response.headers["fh-conversation-id"])[0]
