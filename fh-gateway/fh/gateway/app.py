import os

from authlib.integrations.starlette_client import OAuth
from fastapi import FastAPI, Request
from fastapi.param_functions import Depends
from fastapi_cloudauth.auth0 import Auth0, Auth0Claims, Auth0CurrentUser
from fh.gateway.config import Config
from fh.gateway.proxy import proxy_fh_request
from starlette.middleware.sessions import SessionMiddleware

config: Config = Config.from_env()
app = FastAPI()
app.add_middleware(SessionMiddleware, secret_key=config.session_secret)

oauth = OAuth()
oauth.register(
    "auth0",
    client_id=config.auth0.client_id,
    client_secret=config.auth0.client_secret,
    server_metadata_url=config.auth0.well_known_endpoint,
    client_kwargs={
        "scope": "openid profile email",
    },
)


auth0 = Auth0(domain=config.auth0.domain)
get_current_user = Auth0CurrentUser(domain=config.auth0.domain)


# HINT: using @app.route() fails, because it does somehow not resolve the
# dependencies (in this case, the Auth0Claims)
@app.get("/admin/{tail:path}")
@app.post("/admin/{tail:path}")
@app.put("/admin/{tail:path}")
@app.delete("/admin/{tail:path}")
async def admin(request: Request, user: Auth0CurrentUser = Depends(get_current_user)):
    """
    Proxies all requests to the `/admin` endpoint upstream. Requires a logged in user.
    """
    r = await proxy_fh_request(config.core.upstream, request, None)
    return r


# TODO: remove the old stuff
@app.get("/hello/{tail:path}")
@app.post("/hello/{tail:path}")
@app.put("/hello/{tail:path}")
@app.delete("/hello/{tail:path}")
async def conversation(request: Request):
    """
    Proxies all requests to the `/hello` endpoint upstream. This is still some
    left-over from the initial implementation, which cannot be removed, yet.
    """
    r = await proxy_fh_request(config.core.upstream, request)
    return r


@app.get("/conversation/{tail:path}")
async def conversation(request: Request):
    """
    Proxies all requests to the `/conversation` endpoint upstream.
    """
    r = await proxy_fh_request(config.core.upstream, request)
    return r


@app.get("/processor/{tail:path}")
@app.post("/processor/{tail:path}")
@app.put("/processor/{tail:path}")
@app.delete("/processor/{tail:path}")
@app.head("/processor/{tail:path}")
@app.options("/processor/{tail:path}")
async def processor(request: Request):
    """
    Proxies all requests to the `/processor` endpoint upstream.
    """
    r = await proxy_fh_request(config.core.upstream, request)
    return r


@app.get("/auth/auth0")
async def auth_auth0(request: Request):
    """
    OpenID callback endpoint, which authorizes the access token.
    If valid, the `id_token` part is stored as session cookie with name `auth0_token`.
    """
    token = await oauth.auth0.authorize_access_token(request)
    request.session["auth0_token"] = token["id_token"]
    user = await oauth.auth0.parse_id_token(request, token)
    return dict(user)


@app.get("/login")
async def login(request: Request):
    """
    Redirects the user to the auth0 login page.
    """
    redirect_uri = request.url_for("auth_auth0")
    return await oauth.auth0.authorize_redirect(request, redirect_uri)
