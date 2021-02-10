import os
from typing import Optional

from fastapi import FastAPI, Depends, Request
from fastapi_cloudauth.auth0 import Auth0, Auth0CurrentUser, Auth0Claims
from authlib.integrations.starlette_client import OAuth
from starlette.middleware.sessions import SessionMiddleware
from dotenv import load_dotenv

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


# @app.route("/")
# async def root(request: Request):
#     async with httpx.AsyncClient(base_url="http://localhost:3031") as client:
#         if len(request.url.query) > 0:

#         r = await client.request()


@app.route("/admin")
def admin(current_user: Auth0Claims = Depends(get_current_user)):
    return {"Hello": "World"}


@app.get("/user/", dependencies=[Depends(auth0.scope("read:users"))])
def secure_user(current_user: Auth0Claims = Depends(get_current_user)):
    # ID token is valid
    return f"Hello, {current_user.username}"


@app.get("/items/{item_id}")
def read_item(item_id: int, q: Optional[str] = None):
    return {"item_id": item_id, "q": q}


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
