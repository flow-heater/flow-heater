from copy import deepcopy
from typing import Optional

import requests
from fastapi import Request
from fastapi_cloudauth.auth0 import Auth0Claims
from starlette.datastructures import URL
from starlette.responses import Response


from fh.gateway.auth import FhAuth0JwkData

#
async def proxy_fh_request(
    base_url: str, request: Request, user: Optional[FhAuth0JwkData] = None
):
    """
    Proxies http requests to anywhere we like. It takes the base_url parameter
    and patches the given requests url. Headers and body is forwarded upstream
    and returned on response.

    see also: https://github.com/encode/starlette/pull/24/files

    @TODO: use a shared http session
    """
    headers = request.headers.mutablecopy()

    def patch_url(base_url: str, request: Request) -> URL:
        base_parsed = URL(base_url)
        orig = deepcopy(request.url)
        if orig.scheme == "https":
            headers["x-forwarded-proto"] = "https"
        return orig.replace(
            scheme=base_parsed.scheme,
            hostname=base_parsed.hostname,
            port=base_parsed.port,
        )

    if user:
        headers["fh-user-id"] = user.sub

    url = patch_url(base_url, request)
    del headers["host"]
    headers["connection"] = "keep-alive"
    data = await request.body()
    upstream_response = requests.request(
        request.method,
        url=str(url),
        data=data,
        headers=headers,
    )

    return Response(
        upstream_response.content,
        status_code=upstream_response.status_code,
        headers=upstream_response.headers,
    )
