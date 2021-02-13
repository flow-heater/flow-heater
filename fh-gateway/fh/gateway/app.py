import os

from fastapi import FastAPI, Depends, Request
from fastapi_cloudauth.auth0 import Auth0, Auth0CurrentUser, Auth0Claims
from authlib.integrations.starlette_client import OAuth
from starlette.middleware.sessions import SessionMiddleware
from dotenv import load_dotenv
from starlette.responses import JSONResponse

from fh.gateway.proxy import proxy_fh_request


load_dotenv()
app = FastAPI()
app.add_middleware(SessionMiddleware, secret_key=os.getenv("GATEWAY_SESSION_SECRET"))

oauth = OAuth()
oauth.register(
    "auth0",
    client_id=os.getenv("AUTH0_CLIENT_ID"),
    client_secret=os.getenv("AUTH0_CLIENT_SECRET"),
    server_metadata_url=os.getenv("AUTH0_WELL_KNOWN_ENDPOINT"),
    client_kwargs={
        "scope": "openid profile email",
    },
)


auth0 = Auth0(domain=os.getenv("AUTH0_DOMAIN"))
get_current_user = Auth0CurrentUser(domain=os.getenv("AUTH0_DOMAIN"))


# HINT: using @app.route() fails, because it does somehow not resolve the
# dependencies (in this case, the Auth0Claims)
@app.get("/admin/{tail:path}")
@app.post("/admin/{tail:path}")
@app.put("/admin/{tail:path}")
@app.delete("/admin/{tail:path}")
async def admin(request: Request):
    r = await proxy_fh_request(os.getenv("CORE_UPSTREAM"), request, None)
    return JSONResponse(r)


@app.get("/conversation/{tail:path}")
async def conversation(request: Request):
    r = await proxy_fh_request(os.getenv("CORE_UPSTREAM"), request)
    return JSONResponse(r)


@app.get("/processor/{tail:path}")
@app.post("/processor/{tail:path}")
@app.put("/processor/{tail:path}")
@app.delete("/processor/{tail:path}")
@app.head("/processor/{tail:path}")
@app.options("/processor/{tail:path}")
async def processor(request: Request):
    r = await proxy_fh_request(os.getenv("CORE_UPSTREAM"), request)
    return JSONResponse(r)


@app.get("/auth/auth0")
async def auth_auth0(request: Request):
    token = await oauth.auth0.authorize_access_token(request)
    request.session["auth0_token"] = token["id_token"]
    user = await oauth.auth0.parse_id_token(request, token)
    return dict(user)


@app.get("/login")
async def login(request: Request):
    redirect_uri = request.url_for("auth_auth0")
    return await oauth.auth0.authorize_redirect(request, redirect_uri)
