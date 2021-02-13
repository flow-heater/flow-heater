from copy import deepcopy
from typing import Optional

from fastapi import Request
from fastapi_cloudauth.auth0 import Auth0Claims
from starlette.datastructures import URL

import httpx


async def proxy_fh_request(
    base_url: str, request: Request, user: Optional[Auth0Claims] = None
):
    def patch_url(base_url: str, request: Request) -> str:
        base_parsed = URL(base_url)
        orig = deepcopy(request.url)
        if orig.scheme == "https":
            # TODO: set X-Forwarded-Proto header
            pass
        return orig.replace(
            scheme=base_parsed.scheme,
            hostname=base_parsed.hostname,
            port=base_parsed.port,
        )

    async with httpx.AsyncClient() as client:
        url = patch_url(base_url, request)
        response = await client.request(
            request.method, url=str(url), content=await request.body()
        )
        return response.json()
