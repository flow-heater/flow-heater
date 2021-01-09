import re
from pathlib import Path
from typing import Union

import requests


def preprocess_javascript(code: Union[str, Path]) -> str:
    """
    This is a hack to bring in non-standard
    extensions to user space functions.

    Currently, it implements the ``@fh:include()`` directive.

    ``@fh:include()`` can include arbitrary code from
    either the filesystem or from a HTTP resource.

    :param code: The JavaScript code including custom extensions.
    :return:     The resolved pure JavaScript code.
    """

    # Load entrypoint code either from
    # filesystem or obtain it as string.
    basedir = None
    if isinstance(code, Path):
        basedir = code.absolute().parent
        with open(code, "r") as f:
            jscode = f.read()
    else:
        jscode = code

    # Implement the ``@fh:include`` directive.
    if "// @fh:include" in jscode:
        findings = re.findall(r"^(// @fh:include\(\"(.+?)\"\).*)", jscode, re.MULTILINE)
        for finding in findings:
            original, resource_address = finding
            resource = load_resource(basedir, resource_address)
            jscode = jscode.replace(original, resource)

    return jscode


def load_resource(basedir: Path, resource: str) -> str:
    """
    Load an external resource.
    Either from the filesystem or from the Web.
    When loading from the filesystem, it will prevent directory traversal.

    :param basedir:  Path to base directory.
    :param resource: Path to relative resource.
    :return:         The string content of the resource.
    """
    payload = None
    if resource.startswith("http"):
        payload = requests.get(resource).text

    elif basedir is not None:
        file_resource = get_path_safe(basedir, resource)
        if file_resource.exists():
            with open(file_resource, "r") as f:
                payload = f.read()

    else:
        raise NotImplemented(
            f"Method for acquiring resource not implemented: {resource}"
        )

    if payload is None:
        raise ImportError(f"Could not acquire resource {resource}")

    return payload


def get_path_safe(basedir: Path, resource: str) -> Path:
    """
    Resolve path to file.
    Prevent directory traversal.

    See also:
    https://stackoverflow.com/questions/45188708/how-to-prevent-directory-traversal-attack-from-python-code

    :param basedir:  Path pointing to basedir of the addressed resource.
    :param resource: Relative path designating the addressed resource.
    :return:         Safe path to the resource.
    """
    curdir = Path().absolute()
    return basedir.joinpath(resource).resolve().relative_to(curdir)
