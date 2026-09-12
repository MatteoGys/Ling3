"""
config.py — Constantes globales de l'application Linguistique.
"""

import os
import sys

BASE_DIR = os.path.dirname(os.path.abspath(__file__))


def _get_app_data_dir() -> str:
    """Retourne un dossier accessible en écriture pour l'utilisateur courant,
    quelle que soit la plateforme — et même depuis une app packagée en
    lecture seule (App Translocation sur macOS, Program Files en lecture
    seule sous Windows, etc.). Le dossier est créé s'il n'existe pas encore.
    """
    app_name = "Ling3"
    if sys.platform == "darwin":
        base = os.path.expanduser("~/Library/Application Support")
    elif sys.platform == "win32":
        base = os.environ.get("APPDATA", os.path.expanduser("~"))
    else:  # Linux et autres
        base = os.environ.get("XDG_DATA_HOME", os.path.expanduser("~/.local/share"))

    path = os.path.join(base, app_name)
    os.makedirs(path, exist_ok=True)
    return path


SETTINGS_FILE = os.path.join(_get_app_data_dir(), "user_settings.json")

DEFAULT_PROFILE_DATA: dict = {
    "textgrid_path_show": "",
    "textgrid_path_files": [],
    "excel_path": "",
    "separator": "\\",
}