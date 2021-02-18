import os
from dataclasses import dataclass

from dotenv import load_dotenv


@dataclass
class ConfigAuth0:
    """
    Configuration parameters for the authentication information, especially
    Auth0.
    """

    client_id: str
    client_secret: str
    well_known_endpoint: str
    domain: str


@dataclass
class ConfigCore:
    """
    Configuration parameters for the upstream core.
    """

    port: int
    host: str

    @property
    def upstream(self) -> str:
        return f"http://{self.host}:{self.port}"


@dataclass
class Config:
    """
    Simple structured config wrapper around all config options we provide. There
    are no default values on purpose. Default value handling should be done in
    the `from_env()` classmethod.
    """

    session_secret: str
    port: int
    auth0: ConfigAuth0
    core: ConfigCore

    @classmethod
    def from_env(cls):
        """
        Prepares a full configuration object. Loads all values from the
        environment, but uses `load_dotenv()` to fetch possible defaults from
        `.env` files. The first match wins in the following order:

        1. Environment Variables
        2. .env files
        3. Fallback default values using `os.getenv()`.

        For some variables, the `os.environ[]` accessor is used, to provoke
        exceptions and abort. In such cases, explicit failure is desired, to
        ensure things are explicitly configured. Sensible defaults are needed,
        but it's not the obligation in this layer.
        """
        load_dotenv()
        return Config(
            session_secret=os.environ["GATEWAY_SESSION_SECRET"],
            port=int(os.getenv("GATEWAY_PORT", 3031)),
            core=ConfigCore(
                port=int(os.getenv("CORE_PORT", 3030)),
                host=os.environ["CORE_HOST"],
            ),
            auth0=ConfigAuth0(
                client_id=os.environ["AUTH0_CLIENT_ID"],
                client_secret=os.environ["AUTH0_CLIENT_SECRET"],
                well_known_endpoint=os.environ["AUTH0_WELL_KNOWN_ENDPOINT"],
                domain=os.environ["AUTH0_DOMAIN"],
            ),
        )
