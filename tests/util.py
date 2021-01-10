from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple, Union
from pathlib import Path
from typing import Union

import requests
from dacite import from_dict

from tests.test_admin_processor import RequestProcessor, create_request_processor


@dataclass
class AuditItem:
    id: str
    kind: str
    created_at: str
    conversation_id: str
    payload: Union[str, Dict]
    inc: Optional[int]
    request_id: Optional[str]


@dataclass
class RequestConversation:
    id: str
    request_processor_id: str
    created_at: str
    audit_items: List[AuditItem]


def read_code(filename_or_code: Union[Path, str]) -> str:
    if isinstance(filename_or_code, Path):
        with open(filename_or_code, "r") as f:
            return f.read()
    else:
        return filename_or_code


def create_processor(code: str):

    rp = RequestProcessor(
        id=None,
        name="testing",
        runtime="v8",
        language="javascript",
        code=code,
    )

    response = create_request_processor(rp)
    rp_id = response.json()["id"]

    return rp_id


def run_processor(
    identifier, method="get", prelude=False, **kwargs
) -> requests.Response:
    path = "run" if not prelude else "run_with_prelude"

    response = requests.request(
        method, f"http://localhost:3030/processor/{identifier}/{path}", **kwargs
    )

    if response.status_code < 300:
        assert "fh-conversation-id" in response.headers

    return response


def execute(filename_or_code: Union[Path, str], method="get", prelude=False, **kwargs):

    code = read_code(filename_or_code)

    identifier = create_processor(code)
    response = run_processor(identifier, method=method, prelude=prelude, **kwargs)

    return response


def get_request_conversation(
    conversation_id: str,
) -> Tuple[RequestConversation, requests.Response]:
    response = requests.get(f"http://localhost:3030/conversation/{conversation_id}")
    return (from_dict(data_class=RequestConversation, data=response.json()), response)
