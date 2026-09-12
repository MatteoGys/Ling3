"""
main.py — Point d'entrée de l'application Linguistique (Flet).
"""

import flet as ft

from backend import SettingsManager
from ui.views.chevauchement_view import chevauchement
from ui.views.modification_TextGrid_view import Modification_TextGrid
from ui.views.token_view import Tokenisation_class
from ui.views.concordancier_view import ConcordancierView
from ui.views.stats_view import StatsView
from ui.views.settings_view import SettingsView
from ui.views.info_view import InfoView
from ui.theme import C, set_app_theme


NAV_ITEMS: list[tuple[str, str | ft.IconData, str]] = [
    ("onglet1",  ft.Icons.SEARCH_OFF,                   "Chevauchements"),
    ("onglet2",  ft.Icons.SWAP_HORIZ,                   "Modifications"),
    ("onglet3",  ft.Icons.FORMAT_LIST_NUMBERED_RTL,     "Tokenisation"),
    ("onglet4",  ft.Icons.PLAYLIST_ADD_CHECK,           "Concordancier"),
    ("stats",    ft.Icons.LEADERBOARD,                  "Statistiques"),
]

DEFAULT_VIEW = "onglet1"


def main(page: ft.Page) -> None:
    page.title = "Ling3"
    page.theme_mode = ft.ThemeMode.SYSTEM
    page.bgcolor = "#0E0E0E"
    page.padding = 0
    page.window.width = 1350
    page.window.height = 800
    page.window.min_width = 960
    page.window.min_height = 620
    page.window.title_bar_hidden = False
    page.run_task(page.window.center)

    current_theme = ["light"]

    sm = SettingsManager()

    fp_textgrid = ft.FilePicker()
    fp_excel = ft.FilePicker()
    fp_trs = ft.FilePicker()

    views: dict[str, object] = {
        "onglet1":       chevauchement(page, sm),
        "onglet2":       Modification_TextGrid(page, fp_trs, sm),
        "onglet3":       Tokenisation_class(page, sm),
        "onglet4":       ConcordancierView(page, sm),
        "stats":         StatsView(page, sm),
        "settings":      SettingsView(page, sm, fp_textgrid, fp_excel),
        "info":          InfoView(page, sm),
    }

    current_view: list[str] = [DEFAULT_VIEW]
    sidebar_expanded: list[bool] = [True]
    nav_btns: dict[str, ft.TextButton] = {}
    nav_col = ft.Column(spacing=2, expand=True)
    content_area = ft.Container(expand=True, bgcolor=C["main_text"])

    def switch_view(name: str) -> None:
        old = current_view[0]
        if old != name and old in views:
            v = views[old]
            if hasattr(v, "on_hide"):
                v.on_hide()

        current_view[0] = name
        content_area.content = views[name].root
        _build_nav_buttons(sidebar_expanded[0])

        view = views[name]
        if hasattr(view, "on_show"):
            view.on_show()
        page.update()

### SIDEBAR ZONE ###
# SIDEBAR BUTTONS VIEW
    def _build_nav_buttons(expanded: bool) -> None:
        nav_col.controls.clear()
        nav_btns.clear()
        for name, icon_name, label in NAV_ITEMS:
            is_sel = name == current_view[0]
            item_color = C["validation_btn_default"] if is_sel else C["secondary_text"] #couleur texte onglet sélectionné else non sélectionné

            if expanded:
                btn_content = ft.Row(
                    controls=[
                        ft.Icon(
                            icon_name,
                            size=25,
                            color=item_color,
                        ),
                        ft.Text(
                            label,
                            color=item_color,
                            weight=ft.FontWeight.BOLD if is_sel else ft.FontWeight.NORMAL,
                        ),
                    ],
                    spacing=10,
                    alignment=ft.MainAxisAlignment.START,
                )
            else:
                btn_content = ft.Icon(icon_name, size=25, color=item_color)

            btn = ft.Container(
                content=btn_content,
                on_click=lambda _e, n=name: switch_view(n),
                bgcolor=C["bg_container"] if is_sel else "transparent", #couleur onglet sélectionné
                border_radius=10,
                padding=ft.Padding.symmetric(horizontal=14 if expanded else 0, vertical=12),
                alignment=ft.Alignment(0, 0) if not expanded else None,
                width=210 if expanded else 45, #longueur des boutons onglets
                height=45, #hauteur des boutons onglets
            )

            if not is_sel:
                btn.on_hover = lambda e: (
                    setattr(e.control, "bgcolor", C["white"] if e.data == "true" else "transparent"),
                    e.control.update()
                )

            nav_btns[name] = btn
            nav_col.controls.append(btn)

# GESTION SIDEBAR CHANGEMENT VIEW
    def toggle_sidebar(_e) -> None:
        sidebar_expanded[0] = not sidebar_expanded[0]
        exp = sidebar_expanded[0]

        sidebar_wrap.width = 230 if exp else 65
        nav_col.horizontal_alignment = ft.CrossAxisAlignment.START if exp else ft.CrossAxisAlignment.CENTER

        _build_sidebar_header(exp)
        _build_sidebar_footer(exp)
        _build_nav_buttons(exp)
        page.update()

# SIDEBAR HEADER
    ham_btn = ft.Container(
        content=ft.Icon(icon=ft.Icons.MENU, size=22, color=C["main_text"]),
        on_click=toggle_sidebar,
        padding=5,
        bgcolor="transparent",
    )

    sidebar_header = ft.Row(vertical_alignment=ft.CrossAxisAlignment.CENTER)

    def _build_sidebar_header(expanded: bool) -> None:
        sidebar_header.controls.clear()
        if expanded:
            sidebar_header.controls.extend([
                ft.Text("Ling3", size=16, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                ft.Container(expand=True),
                ham_btn,
            ])
        else:
            sidebar_header.controls.append(
                ft.Container(
                    content=ham_btn,
                    alignment=ft.Alignment(0, 0),
                    expand=True,
                    bgcolor="transparent"
                )
            )

# SIDEBAR FOOTER
    btn_settings = ft.Container(
        ft.Icon(
            icon=ft.Icons.SETTINGS,
            size=14,
            color=C["main_text"]
        ),
        alignment = ft.Alignment(0, 0),
        width = 30,
        height = 30,
        bgcolor = C["bg_container"],
        border_radius = 15,
        on_click = lambda _e: switch_view("settings"),
        tooltip = "Paramètres",
    )

    def toggle_theme(_e) -> None:
        """Bascule le thème global, synchronise la palette C et recharche la vue courante."""
        # 1. Inversion du mode
        new_mode = "dark" if current_theme[0] == "light" else "light"
        current_theme[0] = new_mode

        # 2. Mise à jour de la palette dynamique dans ui/theme.py
        set_app_theme(new_mode)

        # 3. Synchronisation avec le moteur Flet & fond de page
        page.theme_mode = ft.ThemeMode.DARK if new_mode == "dark" else ft.ThemeMode.LIGHT
        page.bgcolor = C.get("bg_onglet", C.get("bg_primary", "#F5F5F5"))

        # 4. Réinstanciation globale des vues avec les nouvelles couleurs de C
        nonlocal views
        views = {
            "onglet1": chevauchement(page, sm),
            "onglet2": Modification_TextGrid(page, fp_trs, sm),
            "onglet3": Tokenisation_class(page, sm),
            "onglet4": ConcordancierView(page, sm),
            "stats": StatsView(page),
            "settings": SettingsView(page, sm, fp_textgrid, fp_excel),
            "info": InfoView(page, sm),
        }

        # 5. Mise à jour des conteneurs de structure (Sidebar & Zone centrale)
        sidebar_wrap.bgcolor = C.get("sidebar", "#F0F0F0")
        sidebar_wrap.border = ft.Border.only(
            right=ft.BorderSide(1, C.get("bg_container_border", "#C4C4C4"))
        )
        content_area.bgcolor = C.get("secondary_text", "#888888")

        # 6. Mise à jour groupée des boutons d'action de la sidebar
        ham_btn.content.color = C.get("main_text", "#1A1A1A")

        for btn in (btn_settings, btn_theme, btn_info):
            btn.bgcolor = C.get("bg_container", "#E9E9E9")
            if hasattr(btn, "content") and hasattr(btn.content, "color"):
                btn.content.color = C.get("main_text", "#1A1A1A")

        # 7. Reconstruction des composants visuels de la sidebar
        _build_sidebar_header(sidebar_expanded[0])
        _build_nav_buttons(sidebar_expanded[0])

        # 8. Rechargement de la vue active et déclenchement de son cycle de vie
        active_key = current_view[0]
        if active_key in views:
            active_view = views[active_key]
            content_area.content = active_view.root

            # Exécute on_show() pour recharger les données dynamiques de la vue courante
            if hasattr(active_view, "on_show"):
                active_view.on_show()

        page.update()

    btn_theme = ft.Container(
        content=ft.Icon(
            icon=ft.Icons.BRIGHTNESS_MEDIUM_SHARP,
            size=14,
            color=C["main_text"],
        ),
        alignment=ft.Alignment(0, 0),
        width=30,
        height=30,
        bgcolor=C["bg_container"],
        border_radius=15,
        on_click=toggle_theme,
        tooltip="Changer le thème",
    )

    btn_info = ft.Container(
        content=ft.Icon(
            icon=ft.Icons.INFO_OUTLINED,  # Icône de cercle d'information (i) en contour
            size=14,
            color=C["main_text"]
            ),
        alignment=ft.Alignment(0, 0),
        width=30,
        height=30,
        bgcolor=C["bg_container"],
        border_radius=15,
        on_click=lambda _e: switch_view("info"),
        tooltip="Documentations",
    )

    sidebar_footer = ft.Container()

    def _build_sidebar_footer(expanded: bool) -> None:
        if expanded:
            sidebar_footer.content = ft.Row(
                alignment=ft.MainAxisAlignment.SPACE_BETWEEN,
                vertical_alignment=ft.CrossAxisAlignment.CENTER,
                controls=[
                    ft.Text("v1.2.1 Bêta", size=11, color="#888888"),
                    ft.Row(
                        spacing=6,
                        controls=[btn_settings, btn_theme, btn_info],
                    ),
                ],
            )
        else:
            sidebar_footer.content = ft.Column(
                horizontal_alignment=ft.CrossAxisAlignment.CENTER,
                spacing=4,
                controls=[
                    btn_settings,
                    btn_theme,
                    btn_info,
                ],
            )

# CREATION SIDEBAR
    sidebar_wrap = ft.Container(
        width=230,
        bgcolor=C["sidebar"], #couleur sidebar
        border=ft.Border.only(right=ft.BorderSide(1, C["bg_container_border"])),
        padding=ft.Padding.all(10),
        content=ft.Column(
            spacing=6,
            expand=True,
            controls=[
                sidebar_header,
                ft.Divider(height=1, color=C["divider_color"]),
                nav_col,
                ft.Divider(height=1, color=C["divider_color"]),
                sidebar_footer,
            ],
        ),
    )

    def on_view_pop(_view) -> None:
        page.views.pop()
        page.update()

    page.on_view_pop = on_view_pop

    page.add(
        ft.Row(
            expand=True,
            spacing=0,
            controls=[sidebar_wrap, content_area],
        )
    )

    page.update()

    _build_sidebar_header(True)
    _build_sidebar_footer(True)
    _build_nav_buttons(True)
    switch_view(DEFAULT_VIEW)

if __name__ == "__main__":
    ft.run(main)