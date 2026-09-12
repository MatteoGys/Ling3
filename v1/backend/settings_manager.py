"""
settings_manager.py — Gestion de la configuration utilisateur.
"""

import json
import os
import sys

# Permet l'import que l'on soit dans le package ou à la racine
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import config


class SettingsManager:
    """Charge, expose et sauvegarde la configuration JSON de l'application."""

    def __init__(self) -> None:
        self.settings: dict = self._load()

    def _load(self) -> dict:
        if os.path.exists(config.SETTINGS_FILE):
            try:
                with open(config.SETTINGS_FILE, "r", encoding="utf-8") as fh:
                    return json.load(fh)
            except (json.JSONDecodeError, OSError):
                pass
        return {
            "current_profile": "Default",
            "profiles": {"Default": config.DEFAULT_PROFILE_DATA.copy()},
        }

    def save(self) -> None:
        with open(config.SETTINGS_FILE, "w", encoding="utf-8") as fh:
            json.dump(self.settings, fh, indent=4, ensure_ascii=False)

    def get_current_data(self) -> dict:
        profile = self.settings.get("current_profile", "Default")
        return self.settings["profiles"].get(profile, config.DEFAULT_PROFILE_DATA.copy())

    def set_textgrid_path_show(self, path: str) -> None:
        profile = self.settings.get("current_profile", "Default")
        self.settings["profiles"].setdefault(profile, config.DEFAULT_PROFILE_DATA.copy())
        self.settings["profiles"][profile]["textgrid_path_show"] = path
        self.save()

    def set_textgrid_path_files(self, path: str) -> None:
        profile = self.settings.get("current_profile", "Default")
        self.settings["profiles"].setdefault(profile, config.DEFAULT_PROFILE_DATA.copy())
        self.settings["profiles"][profile]["textgrid_path_files"] = path
        self.save()

    def set_excel_path(self, path: str) -> None:
        profile = self.settings.get("current_profile", "Default")
        self.settings["profiles"].setdefault(profile, config.DEFAULT_PROFILE_DATA.copy())
        self.settings["profiles"][profile]["excel_path"] = path
        self.save()

    def set_separator(self, separator: str) -> None:
        profile = self.settings.get("current_profile", "Default")
        self.settings["profiles"].setdefault(profile, config.DEFAULT_PROFILE_DATA.copy())
        self.settings["profiles"][profile]["separator"] = separator
        self.save()