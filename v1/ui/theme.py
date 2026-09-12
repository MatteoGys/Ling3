"""
theme.py — Palette de couleurs et constructeurs de composants réutilisables.
"""

# ── Palette ───────────────────────────────────────────────────────────────────

LIGHT_THEME: dict[str, str] = {
    "red":                      "#d32f2f",
    "white":                    "#FFFFFF",
    "export_btn_default":       "#388e3c",
    "export_btn_hovered":       "#006400",
    "validation_btn_default":   "#00A2ED",
    "validation_btn_hovered":   "#1985C1",
    "main_text":                "#1A1A1A",
    "secondary_text":           "#888888",
    "bg_onglet":                "#F5F5F5",
    "bg_container":             "#E9E9E9",
    "bg_container_border":      "#C4C4C4",
    "sidebar":                  "#F0F0F0",
    "divider_color":            "#C4C4C4",
}

DARK_THEME: dict[str, str] = {
    "red":                      "#d32f2f", #
    "white":                    "#2b2d30", ##  #2b2d30, #888888, #1e1f22
    "export_btn_default":       "#388e3c", #
    "export_btn_hovered":       "#006400", #
    "validation_btn_default":   "#00A2ED", #
    "validation_btn_hovered":   "#1985C1", #
    "main_text":                "#dfdfdf",
    "secondary_text":           "#a4a4a4", ##
    "bg_onglet":                "#222222",
    "bg_container":             "#282828",
    "bg_container_border":      "#383838",
    "sidebar":                  "#2b2d30",
    "divider_color":            "#dfdfdf",
}

C: dict[str, str] = LIGHT_THEME.copy()

def set_app_theme(mode: str) -> None:
    """
    Met à jour le dictionnaire C en place selon le mode ('light' ou 'dark').
    """
    C.clear()
    if mode == "dark":
        C.update(DARK_THEME)
    else:
        C.update(LIGHT_THEME)

