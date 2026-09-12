"""
onglet_1_view.py — Modèle "Chevauchements" pour les fichiers linguistiques.
"""

import os
import bisect
import flet as ft
from ui.theme import C

class chevauchement:
    """Analyse un fichier .TextGrid et remonte les erreurs de chevauchement."""

    def __init__(self, page: ft.Page, settings_manager=None) -> None:
        self.page = page
        self.sm = settings_manager

        self.status_text = ft.Text("Prêt à analyser.", color=C["secondary_text"], size=14)

        # Grille de défilement pour stocker les lignes de résultats
        self.table_body = ft.Column(spacing=0)

        # Définition des proportions des colonnes pour un alignement parfait (Header <-> Données)
        self.col_flex = {
            "tier": 2,
            "num": 1,
            "time": 2,
            "text": 4,
            "err": 3
        }

        # Construction de l'en-tête personnalisé
        self.table_header = ft.Container(
            padding=ft.Padding.only(left=16, right=16, top=12, bottom=12),
            content=ft.Row(
                controls=[
                    ft.Container(content=ft.Text("Tier (Source)", weight=ft.FontWeight.BOLD, color=C["main_text"]), expand=self.col_flex["tier"]),
                    ft.Container(content=ft.Text("N° Intervalle", weight=ft.FontWeight.BOLD, color=C["main_text"]), expand=self.col_flex["num"]),
                    ft.Container(content=ft.Text("Timecode (x_max)", weight=ft.FontWeight.BOLD, color=C["main_text"]), expand=self.col_flex["time"]),
                    ft.Container(content=ft.Text("Texte Source", weight=ft.FontWeight.BOLD, color=C["main_text"]), expand=self.col_flex["text"]),
                    ft.Container(content=ft.Text("Erreur dans", weight=ft.FontWeight.BOLD, color="#F01A2C"), expand=self.col_flex["err"]),
                ]
            )
        )

        # Assemblage de la table personnalisée
        self.custom_table = ft.Container(
            bgcolor=C["bg_container"],          # Couleur interne infos
            border=ft.Border.all(1, color=C["bg_container_border"]), # Couleur bordure externe
            border_radius=10,
            content=ft.Column(
                spacing=0,
                controls=[
                    self.table_header,
                    # Ligne séparatrice noms colonnes / résultats
                    ft.Divider(height=4, thickness=2, color=C["main_text"]),
                    # Zone de contenu dynamique
                    ft.Container(
                        content=self.table_body,
                        padding=ft.Padding.only(bottom=8)
                    )
                ]
            )
        )

        self.root = ft.Container(
            expand=True,
            bgcolor=C["bg_onglet"],  # Couleur fond
            padding=24,
            content=ft.Column(
                expand=True,
                controls=[
                    ft.Text("Analyse des chevauchements", size=26, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Divider(height=1, color=C["divider_color"]),
                    ft.Row(
                        controls=[
                            ft.Button(
                                "Lancer l'analyse TextGrid",
                                on_click=self.process_textgrid,
                                style=ft.ButtonStyle(
                                    bgcolor={
                                        ft.ControlState.DEFAULT: C["validation_btn_default"],
                                        ft.ControlState.HOVERED: C["validation_btn_hovered"],
                                    },
                                    color=C["white"],
                                    shape=ft.RoundedRectangleBorder(radius=8),
                                ),
                                height=45,
                            ),
                            self.status_text
                        ],
                        spacing=16,
                        vertical_alignment=ft.CrossAxisAlignment.CENTER
                    ),
                    ft.Container(height=10),
                    ft.ListView(
                        expand=True,
                        controls=[self.custom_table]
                    )
                ]
            )
        )

    def _read_text_auto(self, filepath: str) -> str:
        """Lit un fichier texte en détectant son encodage."""
        with open(filepath, 'rb') as f:
            raw = f.read()

        if raw.startswith(b'\xff\xfe') or raw.startswith(b'\xfe\xff'):
            return raw.decode('utf-16')
        if raw.startswith(b'\xef\xbb\xbf'):
            return raw.decode('utf-8-sig')

        try:
            return raw.decode('utf-8')
        except UnicodeDecodeError:
            return raw.decode('utf-16')

    def parse_textgrid(self, filepath: str) -> dict:
        """Parseur séquentiel du TextGrid vers dict."""
        tiers = {}
        current_tier_id = None
        in_interval = False
        current_interval = None

        content = self._read_text_auto(filepath)
        for line in content.splitlines():
            line = line.strip()
            if line.startswith("item [") and not line.startswith("item []"):
                try:
                    current_tier_id = int(line.split("[")[1].split("]")[0])
                    tiers[current_tier_id] = {'name': '', 'intervals': []}
                    in_interval = False
                except ValueError:
                    pass
            elif line.startswith("name =") and not in_interval and current_tier_id is not None:
                tiers[current_tier_id]['name'] = line.split("=", 1)[1].strip().strip('"')
            elif line.startswith("intervals [") and current_tier_id is not None:
                in_interval = True
                current_interval = {}
                tiers[current_tier_id]['intervals'].append(current_interval)
            elif in_interval and current_interval is not None:
                if line.startswith("xmin ="):
                    current_interval['xmin'] = float(line.split("=")[1].strip())
                elif line.startswith("xmax ="):
                    current_interval['xmax'] = float(line.split("=")[1].strip())
                elif line.startswith("text ="):
                    current_interval['text'] = line.split("=", 1)[1].strip().strip('"')
        return tiers

    def process_textgrid(self, _e) -> None:
        """Moteur de comparaison d'intervalles inter-tiers."""
        data = self.sm.get_current_data()
        tg_paths = data.get("textgrid_path_files", "")
        self.nb_error = 0

        # Nettoyage et injection des lignes de résultats personnalisées
        self.table_body.controls.clear()

        if not tg_paths:
            self.status_text.value = "Aucun fichier TextGrid configuré. Veuillez vérifier vos paramètres."
            self.status_text.color = C["red"]
            self.page.update()
            return

        missing_files = []
        error_files = []
        processed_count = 0

        for element in tg_paths:
            if not element or not os.path.exists(element):
                # On ne bloque plus tout le lot pour un seul fichier manquant.
                missing_files.append(element or "(chemin vide)")
                continue

            self.status_text.value = f"Analyse en cours... ({os.path.basename(element)})"
            self.status_text.color = C["main_text"]
            self.page.update()

            try:
                tiers = self.parse_textgrid(element)
            except Exception as e:
                # Idem : une erreur sur un fichier ne doit pas empêcher
                # l'analyse des autres.
                error_files.append(f"{os.path.basename(element)} ({e})")
                continue

            errors = []
            tier_ids = sorted(tiers.keys())

            # Optimisation : les intervalles d'un tier forment une partition du
            # temps (sans recouvrement), donc il existe au plus UN intervalle
            # contenant un instant donné. Plutôt que de parcourir linéairement
            # (O(I)) tous les intervalles de chaque tier cible pour chaque
            # comparaison, on trie une seule fois les intervalles par xmin et on
            # retrouve l'intervalle candidat par recherche dichotomique (O(log I)).
            # Sur un gros TextGrid (plusieurs milliers d'intervalles par tier),
            # ça transforme un O(T² × I²) en O(T² × I × log I).
            sorted_intervals_by_tier = {}
            xmins_by_tier = {}
            for tid in tier_ids:
                ivs = sorted(tiers[tid]['intervals'], key=lambda iv: iv.get('xmin', 0.0))
                sorted_intervals_by_tier[tid] = ivs
                xmins_by_tier[tid] = [iv.get('xmin', 0.0) for iv in ivs]

            # Nom du fichier (sans extension) affiché à côté du numéro de
            # Tier, pour distinguer les fichiers dans le tableau. On utilise
            # os.path.basename/splitext plutôt qu'un découpage manuel : ça
            # gère correctement les extensions de longueur variable (.txt,
            # .TextGrid...) et les deux styles de séparateurs de chemin
            # (l'ancien `element.rsplit("/", 1)[1][:-4]` tronquait mal les
            # extensions de plus de 4 caractères et plantait sous Windows).
            file_label = os.path.splitext(os.path.basename(element))[0]

            for tier_id in tier_ids:
                for idx_int, interval in enumerate(tiers[tier_id]['intervals']):
                    xmax = interval.get('xmax')
                    if xmax is None:
                        continue

                    error_tiers = []
                    for target_tier_id in tier_ids:
                        if target_tier_id == tier_id:
                            continue

                        xmins = xmins_by_tier[target_tier_id]
                        # Dernier intervalle dont xmin <= xmax : seul candidat
                        # pouvant vérifier t_xmin < xmax < t_xmax.
                        pos = bisect.bisect_right(xmins, xmax) - 1
                        if pos < 0:
                            continue

                        t_interval = sorted_intervals_by_tier[target_tier_id][pos]
                        t_xmin = t_interval.get('xmin', 0)
                        t_xmax = t_interval.get('xmax', 0)

                        if t_xmin < xmax < t_xmax and t_interval.get('text', "") != "":
                            error_tiers.append(f"Tier {target_tier_id}")

                    if error_tiers:
                        errors.append({
                            'source_tier': f"Tier {tier_id} {file_label}",
                            'timecode': xmax,
                            'interval_idx': idx_int + 1,
                            'text': interval.get('text', ''),
                            'error_tiers': ", ".join(error_tiers)
                        })

            for err in errors:
                row_control = ft.Container(
                    padding=ft.Padding.only(left=16, right=16, top=10, bottom=10),
                    content=ft.Row(
                        controls=[
                            ft.Container(content=ft.Text(err['source_tier'], color=C["main_text"]), expand=self.col_flex["tier"]),
                            ft.Container(content=ft.Text(str(err['interval_idx']), color=C["main_text"]), expand=self.col_flex["num"]),
                            ft.Container(content=ft.Text(f"{err['timecode']:.4f}", color=C["main_text"]), expand=self.col_flex["time"]),
                            ft.Container(content=ft.Text(err['text'], color=C["main_text"]), expand=self.col_flex["text"]),
                            ft.Container(content=ft.Text(err['error_tiers'], color="#E56872", weight=ft.FontWeight.BOLD), expand=self.col_flex["err"]),
                        ]
                    )
                )

                self.table_body.controls.append(row_control)
                # LES LIGNES SÉPARANT LES RÉSULTATS
                self.table_body.controls.append(ft.Divider(height=1, thickness=1, color="#D6D6D6"))

            self.nb_error += len(errors)
            processed_count += 1

        parts = [f"{self.nb_error} erreur(s) trouvée(s)", f"{processed_count} fichier(s) analysé(s)"]
        if missing_files:
            parts.append(f"{len(missing_files)} introuvable(s)")
        if error_files:
            parts.append(f"{len(error_files)} en erreur")

        self.status_text.value = " · ".join(parts) + "."
        self.status_text.color = C["red"] if (missing_files or error_files) else C["export_btn_default"]
        self.page.update()

    def on_show(self) -> None:
        pass

    def on_hide(self) -> None:
        pass