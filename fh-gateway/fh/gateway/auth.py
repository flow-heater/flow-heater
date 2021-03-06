from fastapi_cloudauth.auth0 import Auth0Claims, Auth0CurrentUser


class FhAuth0JwkData(Auth0Claims):
    sub: str


class FhAuth0CurrentUser(Auth0CurrentUser):
    user_info = FhAuth0JwkData