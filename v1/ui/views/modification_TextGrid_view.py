"""
onglet_2_view.py — Onglet de modification textuelle, conversion TRS, et extraction par pivots.
"""

import os
import re
import flet as ft
from ui.theme import C

class Modification_TextGrid:
    """Vue permettant de traiter, modifier, et convertir des fichiers d'annotation."""

    def __init__(self, page: ft.Page, fp_trs: ft.FilePicker, settings_manager=None) -> None:
        self.page = page
        self.sm = settings_manager

        # ── Outil de sélection de fichiers TRS ────────────────────────────────
        self._in_trs = []
        self.fp_trs = fp_trs


        # ── Composants UI : Conteneur 1 (Conversion TRS) ─────────────────────
        self._lbl_trs_path = ft.Text("Aucun fichier .trs.xml sélectionné", color=C.get("secondary_text", "#888888"), italic=True, expand=True)
        self._btn_browse_trs = ft.Button(
            "Parcourir",
            icon=ft.Icons.FOLDER_OPEN,
            on_click=self._pick_folder_trs,
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
        )

        self._btn_convert_trs = ft.Button(
            "Convertir",
            on_click=self._convert_trs_to_tg,
            style=ft.ButtonStyle(
                bgcolor={
                    ft.ControlState.DEFAULT: C["validation_btn_default"],
                    ft.ControlState.HOVERED: C["validation_btn_hovered"],
                },
                color=C["white"],
                shape=ft.RoundedRectangleBorder(radius=8),
            ),
            height=40,
        )

        # ── Composants UI : Conteneur 2 (Remplacement & Optimisation) ────────
        self._txt_in = ft.TextField(
            hint_text="Texte cible...",
            bgcolor=C.get("white", "#FFFFFF"),
            border_color=C.get("bg_container_border", "#333333"), color=C.get("main_text", "#E0E0E0"),
            expand=True, height=45, content_padding=10
        )
        self._txt_out = ft.TextField(
            hint_text="Substitution...", bgcolor=C.get("white", "#FFFFFF"),
            border_color=C.get("bg_container_border", "#333333"), color=C.get("main_text", "#E0E0E0"),
            expand=True, height=45, content_padding=10
        )
        self._switch_merge = ft.Switch(
            value=True, active_color=C.get("validation_btn_default", "#388E3C")
        )
        self._btn_apply_modifs = ft.Button(
            "Appliquer les modifications",
            on_click=self._execute_transformations,
            style=ft.ButtonStyle(
                bgcolor={
                    ft.ControlState.DEFAULT: C["validation_btn_default"],
                    ft.ControlState.HOVERED: C["validation_btn_hovered"],
                },
                color=C["white"],
                shape=ft.RoundedRectangleBorder(radius=8),
            ),
            height=40,
        )

        # ── Composants UI : Conteneur 3 (Pivots) ─────────────────────────────
        self._txt_pivots = ft.TextField(
            hint_text="Ex: alors\\du coup\\bref", label="Mots pivots",
            bgcolor=C.get("white", "#FFFFFF"), color=C.get("main_text", "#E0E0E0"),
            border_color=C.get("bg_container_border", "#333333"), expand=True, height=45, content_padding=10
        )

        data = self.sm.get_current_data() if self.sm else {}
        self.separator = data.get("separator", "\\")

        self._btn_apply_pivots = ft.Button(
            "Créer le fichier pivot",
            on_click=self._execute_pivot_extraction,
            style=ft.ButtonStyle(
                bgcolor={
                    ft.ControlState.DEFAULT: C["validation_btn_default"],
                    ft.ControlState.HOVERED: C["validation_btn_hovered"],
                },
                color=C["white"],
                shape=ft.RoundedRectangleBorder(radius=8),
            ),
            height=40,
        )

        # ── Barre de Statut ───────────────────────────────────────────────────
        self._status_text = ft.Text("Prêt.", color=C.get("secondary_text", "#888888"), size=14)

        # ── Construction de la vue racine ─────────────────────────────────────
        self.root = ft.Container(
            expand=True,
            bgcolor=C.get("bg_onglet", "#F5F5F5"),
            padding=28,
            content=ft.Column(
                expand=True,
                spacing=16,
                scroll=ft.ScrollMode.AUTO,
                controls=[
                    ft.Text("Outils TextGrid & Transcriber", size=26, weight=ft.FontWeight.BOLD, color=C.get("main_text", "#E0E0E0")),
                    ft.Divider(height=1, color=C.get("bg_container_border", "#333333")),

                    # C1 : Conversion TRS
                    self._build_container(
                        "Formatage .trs.xml en .TextGrid",
                        "Transforme un fichier d'annotation Transcriber en fichier TextGrid (Enregistré sur le Bureau).",
                        ft.Row([self._btn_browse_trs, self._lbl_trs_path, self._btn_convert_trs], alignment=ft.MainAxisAlignment.SPACE_BETWEEN)
                    ),

                    # C2 : Modification et Nettoyage
                    self._build_container(
                        "Remplacement de texte & Optimisation",
                        "Modifie le texte et fusionne les intervalles vides du TextGrid paramétré.",
                        ft.Column([
                            ft.Row([ft.Text("Remplacement :", width=120, color=C.get("main_text", "#E0E0E0")), self._txt_in, self._txt_out]),
                            ft.Row([ft.Text("Fusionner les silences :", width=180, color=C.get("main_text", "#E0E0E0")), self._switch_merge]),
                            ft.Row([ft.Container(expand=True), self._btn_apply_modifs])
                        ])
                    ),

                    # C3 : Tri par Pivots
                    self._build_container(
                        "Extraction par mots pivots",
                        "Ne garde que les pivots et supprime les blancs automatiquement (Sauvegardé avec '_pivot.TextGrid').",
                        ft.Row([self._txt_pivots, self._btn_apply_pivots]),
                    ),

                    self._status_text
                ]
            )
        )

    # ── Helpers UI ───────────────────────────────────────────────────────────

    def _build_container(self, title: str, description: str, content: ft.Control) -> ft.Container:
        return ft.Container(
            bgcolor=C.get("bg_container", "#222222"),
            padding=20,
            border_radius=12,
            border=ft.Border.all(1, color=C.get("bg_container_border", "#333333")),
            content=ft.Column([
                ft.Text(title, size=16, weight=ft.FontWeight.BOLD, color=C.get("main_text", "#E0E0E0")),
                ft.Text(description, color=C.get("secondary_text", "#888888"), size=13),
                ft.Container(height=4),
                content
            ])
        )

    def _update_status(self, text: str, is_error: bool = False, is_success: bool = False) -> None:
        self._status_text.value = text
        if is_error:
            self._status_text.color = C.get("red", "#D32F2F")
        elif is_success:
            self._status_text.color = C.get("green", "#388E3C")
        else:
            self._status_text.color = C.get("secondary_text", "#888888")
        self.page.update()

    # ── Logique : TRS vers TextGrid (Traduit de Perl) ────────────────────────

    async def _pick_folder_trs(self, _e) -> None:
        files = await self.fp_trs.pick_files(
            dialog_title="Sélectionner un fichier .trs.xml",
            file_type=ft.FilePickerFileType.CUSTOM,
            allowed_extensions=["trs", "xml"],
            allow_multiple=True,
        )
        if files:
            for file in files:
                self._in_trs.append(file.path)

    def _convert_trs_to_tg(self, _e):
        if not self._in_trs:
            #self._lbl_trs_path("⚠️ Veuillez sélectionner un fichier .trs.xml valide.", color="red")
            self.page.update()
            return

        for element in self._in_trs:
            if not os.path.exists(element):
                pass
            else:
                base_name = os.path.splitext(os.path.basename(element))[0]
                # Suppression de la double extension si présente (ex: fichier.trs.xml -> fichier)
                if base_name.endswith(".trs"):
                    base_name = base_name[:-4]

                desktop_path = os.path.join(os.path.expanduser("~"), "Desktop")
                out_path = os.path.join(desktop_path, f"{base_name}.TextGrid")

                try:
                    self._update_status("Conversion en cours...")
                    self._parse_and_convert_trs(element, out_path)
                    self._update_status(f"✅ Conversion réussie ! Enregistré sur le bureau : {base_name}.TextGrid", is_success=True)
                except Exception as ex:
                    self._update_status(f"⚠️ Erreur lors de la conversion : {str(ex)}", is_error=True)

    def _parse_and_convert_trs(self, trs_path: str, out_path: str) -> None:
        """Logique native Python traduisant le script Perl TRS to TextGrid."""
        speakers, tiers, utterances = {}, {}, {}
        start_time, end_time = 100000.0, 0.0

        with open(trs_path, "r", encoding="utf-8", errors="ignore") as f:
            lines = f.readlines()

        in_turn, utt_needs_end = False, False
        turn_start, turn_end = 0.0, 0.0
        active_spkrs, cur_spkr = "", ""
        turn_spkrs = []

        re_spk = re.compile(r'<Speaker.+?id="(.+?)".+?name="(.+?)"')
        re_sec = re.compile(r'<Section.+?startTime="(.+?)".+?endTime="(.+?)"')
        re_turn = re.compile(r'<Turn.+?speaker="(.+?)".+?startTime="(.+?)".+?endTime="(.+?)"')
        re_sync = re.compile(r'<Sync time="(.+?)"')
        re_who = re.compile(r'<Who nb="(\d+?)"')

        for raw_line in lines:
            line = raw_line.strip()

            m_spk = re_spk.search(line)
            if m_spk:
                speakers[m_spk.group(1)] = m_spk.group(2)
                tiers[m_spk.group(1)] = []
                utterances[m_spk.group(1)] = []
                continue

            m_sec = re_sec.search(line)
            if m_sec:
                start_time = min(start_time, float(m_sec.group(1)))
                end_time = max(end_time, float(m_sec.group(2)))
                continue

            m_turn = re_turn.search(line)
            if m_turn:
                in_turn = True
                active_spkrs = m_turn.group(1)
                turn_start = float(m_turn.group(2))
                turn_end = float(m_turn.group(3))
                turn_spkrs = active_spkrs.split()
                cur_spkr = turn_spkrs[0] if turn_spkrs else ""
                continue

            if "</Turn>" in line:
                if utt_needs_end:
                    utt_needs_end = False
                    for s in turn_spkrs:
                        if utterances.get(s): utterances[s][-1]['end'] = turn_end
                in_turn = False
                continue

            if not in_turn: continue

            m_sync = re_sync.search(line)
            if m_sync:
                if utt_needs_end:
                    utt_needs_end = False
                    for s in turn_spkrs:
                        if utterances.get(s): utterances[s][-1]['end'] = float(m_sync.group(1))
                t_val = float(m_sync.group(1))
                for s in turn_spkrs:
                    if s not in utterances: utterances[s] = []
                    utterances[s].append({'start': t_val, 'text': '', 'end': 0.0})
                utt_needs_end = True
                continue

            m_who = re_who.search(line)
            if m_who:
                idx = int(m_who.group(1)) - 1
                if idx < len(turn_spkrs): cur_spkr = turn_spkrs[idx]
                continue

            if not line.startswith('<'):
                text = raw_line.replace('\r', '').replace('\n', '').replace('"', "'").strip()
                if not cur_spkr: cur_spkr = active_spkrs
                if text and cur_spkr in utterances and utterances[cur_spkr]:
                    utterances[cur_spkr][-1]['text'] += text + " "

        # Buffering avec des silences
        for s_id in speakers:
            last_time = start_time
            for utt in utterances.get(s_id, []):
                if last_time < utt['start']:
                    tiers[s_id].append({'start': last_time, 'end': utt['start'], 'text': ''})
                tiers[s_id].append({'start': utt['start'], 'end': utt['end'], 'text': utt['text'].strip()})
                last_time = utt['end']
            if last_time < end_time:
                tiers[s_id].append({'start': last_time, 'end': end_time, 'text': ''})

        # Ecriture TextGrid
        with open(out_path, "w", encoding="utf-8") as out:
            out.write('File type = "ooTextFile"\nObject class = "TextGrid"\n\n')
            out.write(f'xmin = {start_time}\nxmax = {end_time}\ntiers? <exists>\nsize = {len(speakers)}\nitem []:\n')
            for i, s_id in enumerate(sorted(speakers.keys())):
                out.write(f'    item [{i+1}]:\n        class = "IntervalTier"\n')
                out.write(f'        name = "{speakers[s_id]}"\n        xmin = {start_time}\n        xmax = {end_time}\n')
                out.write(f'        intervals: size = {len(tiers[s_id])}\n')
                for j, interval in enumerate(tiers[s_id]):
                    out.write(f'        intervals [{j+1}]:\n            xmin = {interval["start"]}\n')
                    out.write(f'            xmax = {interval["end"]}\n            text = "{interval["text"]}"\n')

    # ── Moteur d'Analyse et de Modification (TextGrid) ───────────────────────
    def _execute_transformations(self, _e) -> None:
        data = self.sm.get_current_data()
        tg_path = data.get("textgrid_path_files", "")
        replace_count = 0
        merged_count = 0
        if not tg_path: return

        for element in tg_path:
            if not element or not os.path.exists(element):
                return

            header, tiers = self._parse_textgrid(element)
            if not header: return

            self.text_in = (self._txt_in.value or "").strip()
            text_out = (self._txt_out.value or "").strip()

            # Remplacement
            if self.text_in:
                for tier in tiers:
                    for interval in tier["intervals"]:
                        if "text" in interval["props"]:
                            pure_text = interval["props"]["text"].strip('"')
                            if self.text_in in pure_text:
                                replace_count += pure_text.count(self.text_in)
                                pure_text = pure_text.replace(self.text_in, text_out)
                                interval["props"]["text"] = f'"{pure_text}"'

            # Fusion des silences
            if self._switch_merge.value:
                merged_count = self._merge_empty_intervals(tiers)

            self._write_textgrid(element, header, tiers)

        summary = "Fichier mis à jour. "
        if self.text_in: summary += f"{replace_count} texte(s) modifié(s). "
        if self._switch_merge.value: summary += f"{merged_count} intervalle(s) fusionné(s)."
        self._update_status(summary, is_success=True)


    def _execute_pivot_extraction(self, _e) -> None:
        data = self.sm.get_current_data()
        tg_path = data.get("textgrid_path_files", "")
        pivot_count = 0
        if not tg_path: return

        for element in tg_path:
            pivots_raw = (self._txt_pivots.value or "").strip()
            if not pivots_raw:
                self._update_status("⚠️ Veuillez définir au moins un pivot.", is_error=True)
                return

            pivots = [p.strip() for p in pivots_raw.split(self.separator) if p.strip()]

            header, tiers = self._parse_textgrid(element)
            if not header: return

            # Filtrage par pivot
            for tier in tiers:
                for interval in tier["intervals"]:
                    if "text" in interval["props"]:
                        pure_text = interval["props"]["text"].strip('"')
                        found_pivot = next((p for p in pivots if p in pure_text), None)

                        if found_pivot:
                            interval["props"]["text"] = f'"{found_pivot}"'
                            pivot_count += 1
                        else:
                            interval["props"]["text"] = '""'

            # Fusion obligatoire des silences
            self._merge_empty_intervals(tiers)

            # Sauvegarde
            base_dir = str(os.path.dirname(element))
            base_name = os.path.splitext(os.path.basename(element))[0]
            out_path = os.path.join(base_dir, f"{base_name}_pivot.TextGrid")

            self._write_textgrid(out_path, header, tiers)
        self._update_status(f"Création(s) réussie(s) ! {pivot_count} pivot(s) isolé(s). Fichier(s) X_pivot.TextGrid", is_success=True)

    # ── Helpers de manipulation TextGrid (Factorisation) ─────────────────────

    def _parse_textgrid(self, path: str):
        content = None
        for enc in ["utf-8-sig", "utf-16", "utf-8", "latin-1"]:
            try:
                with open(path, "r", encoding=enc) as f:
                    content = f.read()
                break
            except UnicodeDecodeError:
                continue

        if content is None:
            self._update_status("Erreur d'encodage TextGrid.", is_error=True)
            return None, None

        lines = content.splitlines()
        header, tiers = [], []
        current_tier, current_interval = None, None
        header_done = False

        tier_idx_pat = re.compile(r"item\s*\[\s*(\d+)\s*\]\s*:")
        interval_idx_pat = re.compile(r"intervals\s*\[\s*(\d+)\s*\]\s*:")

        for line in lines:
            stripped = line.strip()
            if not header_done:
                header.append(line)
                if stripped.startswith("item []"): header_done = True
                continue

            if tier_idx_pat.search(stripped):
                current_tier = {"indent": line.split("item")[0], "props": [], "intervals": []}
                tiers.append(current_tier)
                current_interval = None
                continue

            if current_tier is not None:
                if interval_idx_pat.search(stripped):
                    current_interval = {"indent": line.split("intervals")[0], "props": {}}
                    current_tier["intervals"].append(current_interval)
                    continue

                if current_interval is not None:
                    if "=" in stripped:
                        k, v = stripped.split("=", 1)
                        current_interval["props"][k.strip()] = v.strip()
                else:
                    if "intervals: size" not in stripped:
                        current_tier["props"].append(line)

        return header, tiers

    def _merge_empty_intervals(self, tiers: list) -> int:
        merged_count = 0
        for tier in tiers:
            old_intervals = tier["intervals"]
            cleaned_intervals = []

            for item in old_intervals:
                if not cleaned_intervals:
                    cleaned_intervals.append(item)
                else:
                    last_item = cleaned_intervals[-1]
                    last_txt = last_item["props"].get("text", '""').strip('"').strip()
                    curr_txt = item["props"].get("text", '""').strip('"').strip()

                    if last_txt == "" and curr_txt == "":
                        last_item["props"]["xmax"] = item["props"]["xmax"]
                        merged_count += 1
                    else:
                        cleaned_intervals.append(item)

            tier["intervals"] = cleaned_intervals
        return merged_count

    def _write_textgrid(self, path: str, header: list, tiers: list) -> None:
        output_buffer = []
        output_buffer.extend(header)

        for i, tier in enumerate(tiers, start=1):
            output_buffer.append(f"{tier['indent']}item [{i}]:")
            for prop_line in tier["props"]:
                output_buffer.append(prop_line)

            size_indent = tier["indent"] + "    "
            output_buffer.append(f"{size_indent}intervals: size = {len(tier['intervals'])}")

            for j, interval in enumerate(tier["intervals"], start=1):
                int_indent = interval["indent"]
                output_buffer.append(f"{int_indent}intervals [{j}]:")
                sub_indent = int_indent + "    "
                output_buffer.append(f"{sub_indent}xmin = {interval['props'].get('xmin')}")
                output_buffer.append(f"{sub_indent}xmax = {interval['props'].get('xmax')}")
                output_buffer.append(f"{sub_indent}text = {interval['props'].get('text', '\"\"')}")

        with open(path, "w", encoding="utf-8") as f:
            f.write("\n".join(output_buffer))

    # ── Cycle de Vie ─────────────────────────────────────────────────────────

    def on_show(self) -> None:
        self._status_text.value = "Prêt."
        self._status_text.color = C.get("secondary_text", "#888888")
        self.page.update()

    def on_hide(self) -> None:
        pass