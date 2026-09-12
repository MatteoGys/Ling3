"""
info_view.py — Vue « Informations & ReadMe » de l'application.
"""

import flet as ft
from ui.theme import C

class InfoView:
    """Affiche la documentation, le guide d'utilisation et les informations de version."""

    def __init__(self, page: ft.Page, settings_manager=None) -> None:
        self.page = page
        self.sm = settings_manager

        self.root = ft.Container(
            expand=True,
            bgcolor=C["bg_onglet"],
            padding=32,
            content=ft.Column(
                expand=True,
                scroll=ft.ScrollMode.AUTO,
                spacing=20,
                controls=[
                    ft.Text("Documentations", size=26, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Divider(height=1, color=C["divider_color"]),

                    ft.Text("Navigation", size=22, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Text(
                        "Sur la gauche de l'application se trouve le menu de navigation. De haut en bas : "
                        "\nEn haut se trouvent le nom de l'application et le bouton du menu (☰) vous permettant de le rétracter et "
                        "de gagner en visibilité. "
                        "\nAu milieu se trouvent les cinq onglets vers permettant d'accéder à différents outils. "
                        "Leur utilisation sera développée plus bas."
                        "\nEn bas du menu se trouve la version de l'application, ainsi que trois boutons. Le premier vous "
                        "permet d'accéder aux paramètres (utilisation également développée plus bas), le deuxième de "
                        "changer de thème (clair/sombre), le dernier permet d'accéder à la documentation (page actuelle)." ,
                            size=18, text_align=ft.TextAlign.JUSTIFY, color=C["main_text"]),

                    ft.Text("Paramètres", size=22, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Text(
                        "Afin de pouvoir utiliser l'application, la première étape est de configurer l'application. "
                        "Pour se faire, rendez-vous dans les paramètres. Vous pouvez y modifier trois variables. "
                        "\nLa première zone de texte vous demande d'insérer le(s) fichier(s) que "
                        "vous souhaitez traiter. Le bouton \"Parcourir\" vous permet d'ouvrir une fenêtre et de "
                        "sélectionner le fichier facilement en vous baladant dans vos fichiers. Deux formats de fichier "
                        "sont acceptés : .TextGrid et .txt. Les cinq onglets ont besoin de ce paramètre pour fonctionner."
                        "\nLa deuxième zone fonctionne de la même manière que la première. Vous pouvez y insérer le "
                        "fichier Excel que vous souhaitez utiliser. Il n'accepte qu'un seul fichier à la fois et il "
                        "n'accepte que le format .xlsx (Excel). Ce paramètre est requis des tableaux dans Tokenisation, "
                        "Concordancier et Statistiques."
                        "\nLe dernier paramètre est un séparateur. Il est d'origine mis un backslash (\\). Vous pouvez "
                        "toutefois le modifier à votre convenance. Ce paramètre est utilisé dans Modifications et dans Concordancier."
                        "Lorsque vous devez choisir un pivot, vous avez la possibilité d'en sélectionner plusieurs à la "
                        "fois. Pour ce faire, séparez les mots par le séparateur choisi, sans espace entre les mots. Les "
                        "claviers d'ordinateur et les conventions d'annotations n'étant pas universelles, vous avez la "
                        "liberté de choisir le séparateur qui vous convient.",
                        size=18, color=C["main_text"]),

                    ft.Text("Chevauchements", size=22, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Text(
                        "Cet onglet sert à vérifier les chevauchements dans votre TextGrid. Une fois le(s) chemin(s) enregistré(s)"
                        "dans les paramètres, cliquer sur le bouton bleu \"Lancer l'analyse TextGrid\". "
                        "\nSi des chevauchements sont mal réalisés, ils apparaîtront dans le tableau. Dans le cas contraire, "
                        "félicitations. "
                        "\nCinq informations vous sont données pour chaque chevauchements mal réaliés : la tier source "
                        "accompagné du nom du fichier, le numéro "
                        "de l'intervalle sur cette tier, le timecode, le texte de l'intervalle, et par rapport à quelle tier "
                        "l'erreur est relative. Tous ces paramètres devraient faciliter la recherche de l'intervalle dans "
                        "votre fichier.",
                        size=18, text_align=ft.TextAlign.JUSTIFY, color=C["main_text"]),

                    ft.Text("Modifications", size=22, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Text(
                        "Cet onglet vous permet d'effectuer toutes les modifications utiles à vos fichiers afin d'en "
                        "extraire efficacement les données ultérieurement."
                        "\nLa première zone \"Formatage .trs.xml en .TextGrid\" sert à la conversion des fichiers du CFPP."
                        "Le fichier de sortie est mis sur votre bureau. Plusieurs fichiers peuvent être converti à la fois. "
                        "La conversion se base sur le code Pearl réalisé dans le cadre du projet LingTools de l'Université "
                        "de l'Oregon, adapté en Python. Pour plus d'information sur le projet : "
                        "https://lingtools.uoregon.edu/tools/trans_to_praat.php"
                        "\nLa deuxième zone \"Remplacement de texte & Optimisation\" vous permet, d'abord, de remplacer "
                        "du texte dans tout le fichier, d'un seul coup. Ceci peut être utile si vous faites une erreur à "
                        "travers tout le document (p.e. une erreur de convention : euhm -> hum). Vous y trouverez "
                        "également un bouton qui vous permet, si vous l'activez, de supprimer les frontières inutiles "
                        "qui séparent deux intervalles vides. Cette fonctionnalité est purement esthétique."
                        "\nLa troisième zone \"Extraction par mots pivots\" est la primordiale pour exploiter pleinement "
                        "les fonctionnalités du concordancier, certaines étiquettes nécessitant que les pivots aient leur "
                        "intervalle propres. Inscrivez les pivots que vous souhaitez garder. Un nouveau fichier .TextGrid "
                        "se trouve sur votre bureau. Seuls les intervalles avec vos pivots sont gardés. Replacer les "
                        "frontières correctement et ajouter les mots environnants vos pivots dans de nouveaux intervalles "
                        "adjacents.",
                        size=18, text_align=ft.TextAlign.JUSTIFY, color=C["main_text"]),

                    ft.Text("Tokenisation", size=22, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Text(
                        "Cet onglet sert à rendre compte de tous les tokens de votre fichier TextGrid, de compter les occurences"
                        "et de vous renvoyer le résultat sous forme de tableau. "
                        "\nTapez le token que vous chercher dans la barre de recherche. Le tableau se met à jour "
                        "instantanément. En cas de problème, un bouton bleu \"Rafraîchir\" est à votre disposition. "
                        "\nLe tableau affiche deux colonnes d'origine : Tokens, affichant le token, et Occ. Tot., "
                        "affichant le nombre d'occurences total du token. "
                        "\nLe bouton \"Tiers\" permet l'ajout des colonnes. En cliquant dessus, le bouton affichera tous "
                        "les noms des tiers de votre fichier TextGrid. En cliquand dessus, vous ajoutez une colonne comptant "
                        "le nombre d'occurences du token propre à la tier. Cliquer à nouveau sur le nom d'une tier pour la supprimer."
                        "\nToutes les colonnes permettent un tri relatif à la colonne en cliquant sur le triangle à côté du nom. "
                        "La colonne Token trie par odre alphabétique.Les autres colonnes trient par nombre d'occurences. "
                        "La colonne triant est celle avec un triangle blanc."
                        "\nFinalement, en bas à gauche se trouve un bouton \"Export Excel\" vous permettant d'exporter "
                        "exactement le tableau affiché dans le fichier Excel.",
                        size=18, text_align=ft.TextAlign.JUSTIFY, color=C["main_text"]),

                    ft.Text("Concordancier", size=22, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Text("Cet onglet permet la création d'un concordancier sur mesure ou d'extraire les données propres "
                            "à vos pivots. Dans le premier cas, vous aurez besoin du fichier générique, contenant la transcription "
                            "complète. Dans le second, vous aurez besoin du  fichier réalisé dans l'onglet Modification, "
                            "ne contenant que vos pivots."
                            "\nDans la barre de recherche, insérez le ou "
                            "les mots que vous souhaitez voir apparaître dans votre concordancier. Pour rappel, si vous "
                            "mettez plusieurs mots, n'insérez pas d'espace et placez le séparateur (variable présente dans les paramètres)"
                            "entre les mots."
                            "\nVous pouvez ensuite choisir le nombre de mot dans le contexte droit et gauche. D'origine, "
                            "le contexte a une longueur de 5."
                            "\nDans la zone \"Ordre des colones\", vous trouverez des étiquettes à la verticale. D'origine "
                            "se trouvent quatre colonnes déjà placés. "
                            "\nVous pouvez drag and drop les colonnes pour les déplacer. "
                            "\nCliquez sur les colonnes disponibles se trouvant à droite pour les ajouter. "
                            "\nCliquez sur la croix rouge sous une étiquette pour la supprimer et la renvoyer dans "
                            "\"Colonnes disponisbles\"."
                            "\nLe bouton \"Rénitialiser l'ordre\" permet de revenir aux quatre colonnes originelles."
                            "\nLe bouton \"Générer le fichier Excel\" permet de créer le concordancier dans le fichier Excel"
                            "avec les colonnes et leurs ordres."
                            "\nLes étiquettes sont les suivantes : "
                            "\n• ID est le numéro attribué à une occurence. Elles sont numérotées tier par tier."
                            "\n• Contexte gauche et contexte droit sont les contextes précédents et suivants le pivot."
                            "\n• Pivot est le ou les mots que vous étudiez."
                            "\n• Tier Nom, Tier Numéro et Tier Lettre est la tier de provenance de l'occurence. Vous pouvez "
                            "anonymisé en numérotant ou en attribuant une lettre."
                            "\nNote : les étiquettes suivantes doivent être utilisés avec des pivots disposant de leur intervalle propre."
                            "\n• x_min et x_max sont les timecodes de début et de fin de l'occurence."
                            "\n• durée_occurence est la durée de l'occurence, la différence entre x_max et x_min."
                            "\n• POS_PAUSES_VIDES permet d'étudier l'impact de l'environnement immédiat. Si l'occurence"
                            "est uniquement précédée d'une pause vide, l'occurence est en position initiale. Si l'occurence "
                            "est uniquement suivie d'une pause vide, l'occurence est en position finale. Si l'occurence est précédée "
                            "et suivie d'une pause vide, elle est en position isolée. Si l'occurence n'est ni précéde ni "
                            "suivie d'une pause vide, l'occurence est en positon finale.",
                            size=18, text_align=ft.TextAlign.JUSTIFY, color=C["main_text"]),

                    ft.Text("Statistiques", size=22, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Text("Cet onglet permet d'étudier l'environnement par rapport à un pivot grâce aux nombres d'occurences."
                            "Dans la barre de recherche, insérez un unique mot. "
                            "\nÀ droite de la barre se trouvent deux boutons : \"Précédent\" et \"Suivant\". Sélectionnez-en"
                            "un. Il vous permet de choisir le sens d'étude. "
                            "\nLe bouton \"Écart(s)\", d'origine à 1, vous permet de choisir la distance en nombre de mots "
                            "depuis le pivot. À 1, vous étudiez le mot suivant. À 2, vous étudiez le deuxième mot après "
                            "le pivot. Vous pouvez choisir plusieurs écarts en séparant les nombres d'une virgule, sans espace "
                            "(p.e. 1,2,4). "
                            "\nLe bouton Tier permet d'ajouter ou de supprimer les colonnes comptant le nombre d'occurence"
                            "pour chaque tier. Elles sont d'origine toutes activées. "
                            "\nAppuyez sur le bouton bleu \"Analyser\" pour lancer l'analyse et afficher le tableau de résultats."
                            "\nLe tableau comporte deux colonnes fixes, auxquelles peuvent s'ajouter les colonnes relatives aux tiers."
                            "\n• La colonne Type affiche le pivot concaténer à son contexte. Un underscore (_) représente "
                            "un mot d'écart. "
                            "\n• La colonne Occ. Tot. comptabilise le nombre d'occurences avec le contexte sur le nombre "
                            "d'occurences totales du pivot (réalisées par l'ensemble des tiers)."
                            "\n• Les colonnes Tiers comptabilise le nombre d'occurence avec le contexte réalisées dans la tier"
                            "sur le nombre d'occurences totales réalisées par la tier."
                            "\nNote : la suppression d'une colonne Tier supprime les occurences dans la colonne Occ. Tot."
                            "\nFinalement, en bas à gauche se trouve un bouton \"Export Excel\" vous permettant d'exporter "
                            "exactement le tableau affiché dans le fichier Excel.",
                            size=18, text_align=ft.TextAlign.JUSTIFY, color=C["main_text"]),

                    # Pied de page d'informations
                    ft.Text("Version 1.2.1 — Développé pour la recherche linguistique."
                            "\nPour toutes autres informations, suggestions ou reports, merci de me contacter à l'adresse "
                            "suivante : matteo.gyselinck@student.outlook.com",
                            size=12, color=C["secondary_text"]),
                ]
            )
        )

    def on_show(self) -> None:
        pass

    def on_hide(self) -> None:
        pass