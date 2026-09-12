"""
settings_view.py — Vue « Paramètres » : Fichier TextGrid et base Excel.
"""

import flet as ft
from ui.theme import C


class SettingsView:
    def __init__(self, page: ft.Page, settings_manager, fp_textgrid: ft.FilePicker, fp_excel: ft.FilePicker) -> None:
        self.page = page
        self.sm = settings_manager

        self._in_textgrid_files = []
        self.fp_textgrid = fp_textgrid
        self.fp_excel = fp_excel

        self._build_controls()

    def _build_controls(self) -> None:
        self._in_textgrid = ft.TextField(
            label="Fichier source .TextGrid ou .txt",
            hint_text="/chemin/vers/votre/fichier.TextGrid",
            bgcolor=C["white"],  # intérieur zone texte
            border_color=C["bg_container_border"],  # bordure extérieur de base
            focused_border_color=C["validation_btn_default"],  # bordure extérieur quand clic
            color=C["main_text"],
            label_style=ft.TextStyle(color=C["secondary_text"]),
            expand=True,
        )
        self._in_excel = ft.TextField(
            label="Fichier de base Excel",
            hint_text="/chemin/vers/votre/fichier.xlsx",
            bgcolor=C["white"],  # intérieur zone texte
            border_color=C["bg_container_border"],  # bordure extérieur de base
            focused_border_color=C["validation_btn_default"],  # bordure extérieur quand clic
            color=C["main_text"],
            label_style=ft.TextStyle(color=C["secondary_text"]),
            expand=True,
        )

        self._in_separator = ft.TextField(
            label="Séparateur",
            hint_text="Ex: \\ ou |",
            value="\\",
            width=150,
            bgcolor=C["white"],
            border_color=C["bg_container_border"],
            focused_border_color=C["validation_btn_default"],
            color=C["main_text"],
            label_style=ft.TextStyle(color=C["secondary_text"]),
        )

        self._status = ft.Text("", size=13)

        self.root = ft.Container(
            expand=True,
            bgcolor=C["bg_onglet"],
            padding=ft.Padding.symmetric(horizontal=40, vertical=32),
            content=ft.Column(
                spacing=20,
                controls=[
                    ft.Text("Paramètres", size=26, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Divider(height=1, color=C["divider_color"]),

                    ft.Text("Fichier .TextGrid ou .txt", size=15, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Row(
                        controls=[
                            self._in_textgrid,
                            ft.Button(
                                "Parcourir",
                                icon=ft.Icons.FOLDER_OPEN,
                                on_click=self._pick_folder_tg,
                                style=ft.ButtonStyle(
                                    bgcolor={
                                        ft.ControlState.DEFAULT: C["validation_btn_default"],
                                        ft.ControlState.HOVERED: C["validation_btn_hovered"],
                                    },
                                    color=C["white"],
                                    shape=ft.RoundedRectangleBorder(radius=8),
                                    side=ft.BorderSide(1, color=C["bg_container_border"]),
                                ),
                                height=54,
                            ),
                        ],
                        spacing=12,
                    ),

                    ft.Divider(height=1, color=C["bg_container_border"]),

                    ft.Text("Fichier Excel", size=15, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Row(
                        controls=[
                            self._in_excel,
                            ft.Button(
                                "Parcourir",
                                icon=ft.Icons.FOLDER_OPEN,
                                on_click=self._pick_folder_xlsx,
                                style=ft.ButtonStyle(
                                    bgcolor={
                                        ft.ControlState.DEFAULT: C["validation_btn_default"],
                                        ft.ControlState.HOVERED: C["validation_btn_hovered"],
                                    },
                                    color=C["white"],
                                    shape=ft.RoundedRectangleBorder(radius=8),
                                    side=ft.BorderSide(1, color=C["bg_container_border"]),
                                ),
                                height=54,
                            ),
                        ],
                        spacing=12,
                    ),

                    ft.Divider(height=1, color=C["bg_container_border"]),

                    ft.Text("Séparateur de mots clés / pivots", size=15, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Row(
                        controls=[self._in_separator],
                        spacing=12,
                    ),

                    ft.Divider(height=1, color=C["bg_container_border"]),

                    ft.Row(
                        spacing=16,
                        controls=[
                            ft.Button(
                                "Enregistrer",
                                on_click=self._save,
                                style=ft.ButtonStyle(
                                    bgcolor={
                                        ft.ControlState.DEFAULT: C["export_btn_default"],
                                        ft.ControlState.HOVERED: C["export_btn_hovered"],
                                    },
                                    color=C["white"],
                                    shape=ft.RoundedRectangleBorder(radius=10),
                                ),
                                height=50, width=190,
                            ),
                        ],
                    ),
                    self._status,
                ],
            ),
        )

    # ── Handlers Asynchrones ─────────────────────────────────────────────────

    async def _pick_folder_tg(self, _e) -> None:
        files = await self.fp_textgrid.pick_files(
            dialog_title="Sélectionner un fichier TextGrid",
            file_type=ft.FilePickerFileType.CUSTOM,
            allowed_extensions=["TextGrid", "txt"],
            allow_multiple=True,
        )
        if files:
            if len(files) > 1:
                self._in_textgrid.value = files[0].path.rsplit("/", 1)[0]
            else:
                self._in_textgrid.value = files[0].path
            self._in_textgrid.update()
            self._in_textgrid_files.clear()
            for file in files:
                self._in_textgrid_files.append(file.path)

    async def _pick_folder_xlsx(self, _e) -> None:
        files = await self.fp_excel.pick_files(
            dialog_title="Sélectionner un fichier Excel",
            file_type=ft.FilePickerFileType.CUSTOM,
            allowed_extensions=["xlsx"],
            allow_multiple=False,
        )
        if files and files[0].path:
            self._in_excel.value = files[0].path
            self._in_excel.update()

    def _save(self, _e) -> None:
        tg_path_show = (self._in_textgrid.value or "").strip()
        tg_path_files = (self._in_textgrid_files)
        ex_path = (self._in_excel.value or "").strip()
        separator = (self._in_separator.value or "").strip()
        changed = False

        if tg_path_show:
            self.sm.set_textgrid_path_show(tg_path_show)
            changed = True
        if tg_path_files:
            self.sm.set_textgrid_path_files(tg_path_files)
        if ex_path:
            self.sm.set_excel_path(ex_path)
            changed = True
        if separator:
            self.sm.set_separator(separator)
            changed = True

        if changed:
            self._status.value = "Paramètres enregistrés avec succès."
            self._status.color = C["export_btn_default"]
        else:
            self._status.value = "Aucune modification à enregistrer."
            self._status.color = C["secondary_text"]
        self._status.update()

    def on_show(self) -> None:
        data = self.sm.get_current_data()
        self._in_textgrid.value = data.get("textgrid_path_show", "")
        self._in_excel.value = data.get("excel_path", "")
        self._in_separator.value = data.get("separator", "\\")
        self._status.value = ""
        try:
            self._in_textgrid.update()
            self._in_excel.update()
            self._in_separator.update()
            self._status.update()
        except Exception:
            pass

    def on_hide(self) -> None:
        pass