//! info.rs (vue) — Remplace ui/views/info_view.py. Page entièrement statique
//! (aucun signal, aucun appel invoke) : simple documentation de l'app.

use leptos::prelude::*;

use crate::icons::{
    IconConcordancier, IconMenu, IconModification, IconOverlap, IconCooccurrences, IconSettings, IconToken,
};

#[component]
pub fn InfoView() -> impl IntoView {
    view! {
        <div class="info-page">
            <div class="info-hero">
                <h1>"Documentation"</h1>
                <p>
                    "Ling3 est un outil d'analyse de fichiers TextGrid (Praat) pensé pour le "
                    "travail linguistique : détection de chevauchements, préparation et conversion "
                    "de fichiers, comptage de tokens, concordancier sur mesure et étude du contexte "
                    "statistique d'un pivot."
                </p>
            </div>

            <div class="info-layout">
                <nav class="info-toc">
                    <a href="#navigation"><IconMenu />"Navigation"</a>
                    <a href="#parametres"><IconSettings />"Paramètres"</a>
                    <a href="#chevauchements"><IconOverlap />"Chevauchements"</a>
                    <a href="#modifications"><IconModification />"Modifications"</a>
                    <a href="#tokenisation"><IconToken />"Tokenisation"</a>
                    <a href="#concordancier"><IconConcordancier />"Concordancier"</a>
                    <a href="#cooccurrences"><IconCooccurrences />"Cooccurrences"</a>
                </nav>

                <div class="info-content">
                    <section id="navigation" class="info-section">
                        <h2><IconMenu />"Navigation"</h2>
                        <p>
                            "La barre latérale à gauche donne accès à tout le reste de l'application. "
                            "En haut, le nom de l'application et le bouton ☰ permettent de la réduire "
                            "pour gagner de la place à l'écran."
                        </p>
                        <p>
                            "Au centre, cinq onglets donnent accès aux outils d'analyse détaillés "
                            "ci-dessous. En bas, trois boutons : l'accès aux Paramètres (à configurer "
                            "en premier), le changement de thème clair/sombre, et cette page de "
                            "documentation."
                        </p>
                    </section>

                    <section id="parametres" class="info-section">
                        <h2><IconSettings />"Paramètres"</h2>
                        <p>
                            "Avant d'utiliser les outils d'analyse, configurez trois éléments dans "
                            "l'onglet Paramètres :"
                        </p>
                        <ul>
                            <li>
                                <strong>"Fichier(s) TextGrid"</strong>
                                " — le ou les fichiers à traiter ("<code>".TextGrid"</code>" ou "
                                <code>".txt"</code>"). Le bouton "<strong>"Parcourir"</strong>
                                " ouvre le sélecteur natif de votre système. Requis par les cinq "
                                "onglets d'analyse."
                            </li>
                            <li>
                                <strong>"Fichier Excel"</strong>
                                " — le classeur "<code>".xlsx"</code>" dans lequel seront écrits les "
                                "tableaux exportés (Tokenisation, Concordancier, Cooccurrences). Un "
                                "seul fichier à la fois."
                            </li>
                            <li>
                                <strong>"Séparateur"</strong>
                                " — utilisé pour distinguer plusieurs pivots saisis dans un même "
                                "champ (par défaut, une barre oblique inversée "<code>"\\"</code>
                                "). Si vous saisissez plusieurs mots pivots, séparez-les par ce "
                                "caractère, sans espace. Utilisé dans Modifications et Concordancier."
                            </li>
                        </ul>
                    </section>

                    <section id="chevauchements" class="info-section">
                        <h2><IconOverlap />"Chevauchements"</h2>
                        <p>
                            "Vérifie qu'aucun intervalle annoté ne chevauche un intervalle d'une "
                            "autre tier au sein d'un même fichier TextGrid."
                        </p>
                        <p>
                            "Une fois vos fichiers configurés, cliquez sur "
                            <strong>"Lancer l'analyse TextGrid"</strong>". Si le tableau reste vide, "
                            "aucun problème n'a été détecté. Sinon, chaque ligne donne cinq "
                            "informations pour localiser rapidement l'intervalle en cause : la tier "
                            "source (avec le nom du fichier), le numéro de l'intervalle, son "
                            "timecode de fin, son texte, et la ou les tiers avec lesquelles il entre "
                            "en conflit."
                        </p>
                    </section>

                    <section id="modifications" class="info-section">
                        <h2><IconModification />"Modifications"</h2>
                        <p>"Regroupe les opérations de préparation des fichiers avant analyse."</p>
                        <ul>
                            <li>
                                <strong>"Conversion TRS → TextGrid"</strong>
                                " — convertit un ou plusieurs fichiers "<code>".trs"</code>
                                " (Transcriber, format du CFPP) en fichiers "<code>".TextGrid"</code>
                                ". Les fichiers convertis sont enregistrés sur le Bureau. "
                                "L'algorithme s'appuie sur le projet LingTools de l'Université de "
                                "l'Oregon (lingtools.uoregon.edu/tools/trans_to_praat.php)."
                            </li>
                            <li>
                                <strong>"Remplacement de texte & fusion des silences"</strong>
                                " — remplace une chaîne de caractères dans tout le fichier en une "
                                "seule opération (utile pour corriger une convention d'annotation "
                                "partout à la fois). La case à cocher permet, en plus ou à la place, "
                                "de fusionner les intervalles de silence consécutifs — un nettoyage "
                                "purement esthétique."
                            </li>
                            <li>
                                <strong>"Extraction par mots pivots"</strong>
                                " — isole, dans un nouveau fichier "<code>"*_pivot.TextGrid"</code>
                                ", uniquement les intervalles contenant l'un de vos pivots ; les "
                                "intervalles adjacents sont fusionnés en silence. Cette étape est "
                                "nécessaire pour utiliser, dans le Concordancier, les colonnes qui "
                                "exigent que le pivot ait son propre intervalle."
                            </li>
                        </ul>
                    </section>

                    <section id="tokenisation" class="info-section">
                        <h2><IconToken />"Tokenisation"</h2>
                        <p>
                            "Comptabilise chaque token (mot) présent dans vos fichiers TextGrid et "
                            "en affiche les occurrences sous forme de tableau."
                        </p>
                        <p>
                            "Tapez un mot dans la barre de recherche pour filtrer le tableau "
                            "instantanément. Le bouton "<strong>"Rafraîchir"</strong>
                            " relance l'analyse depuis vos fichiers."
                        </p>
                        <p>
                            "Le tableau affiche par défaut deux colonnes : "<strong>"Tokens"</strong>
                            " et "<strong>"Occ. Tot."</strong>" (occurrences totales, tous tiers "
                            "confondus). Le bouton "<strong>"Tiers ▾"</strong>" permet d'ajouter, "
                            "tier par tier, une colonne de comptage propre à cette tier."
                        </p>
                        <p>
                            "Chaque en-tête de colonne est cliquable pour trier le tableau : la "
                            "colonne activement triée affiche un triangle plein ("<code>"▲"</code>
                            " croissant, "<code>"▼"</code>" décroissant), les autres un triangle vide."
                        </p>
                        <p>
                            "Le bouton "<strong>"Export Excel"</strong>" exporte exactement le "
                            "tableau affiché à l'instant — recherche et colonnes actives comprises."
                        </p>
                    </section>

                    <section id="concordancier" class="info-section">
                        <h2><IconConcordancier />"Concordancier"</h2>
                        <p>
                            "Génère un concordancier sur mesure dans un fichier Excel : chaque "
                            "occurrence d'un pivot, entourée de son contexte gauche et droit."
                        </p>
                        <p>
                            "Utilisez le fichier TextGrid complet pour un concordancier général, ou "
                            "le fichier "<code>"*_pivot"</code>" créé dans Modifications si vous "
                            "voulez uniquement les occurrences déjà isolées."
                        </p>
                        <p>
                            "Indiquez un ou plusieurs pivots (séparés par le séparateur défini dans "
                            "les Paramètres, sans espace), ainsi que le nombre de mots de contexte à "
                            "gauche et à droite (5 par défaut)."
                        </p>
                        <p>
                            "La section "<strong>"Ordre des colonnes"</strong>" détermine ce qui "
                            "sera exporté, et dans quel ordre : utilisez les flèches "<code>"▲"</code>
                            "/"<code>"▼"</code>" pour réordonner une colonne active, "<code>"✕"</code>
                            " pour la retirer, ou cliquez sur une colonne disponible pour l'ajouter. "
                            <strong>"Réinitialiser l'ordre"</strong>" revient aux quatre colonnes par "
                            "défaut (ID, contexte gauche, pivot, contexte droit)."
                        </p>
                        <p>"Détail des colonnes disponibles :"</p>
                        <ul>
                            <li><strong>"ID"</strong>" — numéro attribué à chaque occurrence."</li>
                            <li>
                                <strong>"Tier Nom / Tier Numéro / Tier Lettre"</strong>
                                " — identifient la tier d'origine ; pratique pour anonymiser en "
                                "gardant un numéro ou une lettre plutôt que le nom réel."
                            </li>
                            <li>
                                <strong>"contexte gauche / pivot / contexte droit"</strong>
                                " — le texte trouvé autour de l'occurrence."
                            </li>
                            <li>
                                <strong>"x_min / x_max / durée_occurence"</strong>
                                " — timecodes de début, de fin et durée. Nécessitent que le pivot "
                                "ait son propre intervalle (voir Extraction par mots pivots)."
                            </li>
                            <li>
                                <strong>"POS_PAUSES_VIDES"</strong>
                                " — position de l'occurrence par rapport aux silences voisins : "
                                <strong>"Initial"</strong>" (précédée d'un silence), "
                                <strong>"Final"</strong>" (suivie d'un silence), "
                                <strong>"Isolé"</strong>" (les deux), "<strong>"Médian"</strong>
                                " (aucun des deux). Nécessite aussi que le pivot ait son propre "
                                "intervalle."
                            </li>
                        </ul>
                        <p>
                            "Le bouton "<strong>"Générer le fichier Excel"</strong>" lance "
                            "l'extraction et écrit directement le résultat dans le fichier Excel "
                            "configuré."
                        </p>
                    </section>

                    <section id="cooccurrences" class="info-section">
                        <h2><IconCooccurrences />"Cooccurrences"</h2>
                        <p>
                            "Étudie l'environnement immédiat d'un pivot en comptant les mots qui "
                            "apparaissent à une certaine distance de lui."
                        </p>
                        <p>
                            "Saisissez un noeud/pivot (un seul mot ou une courte expression), choisissez "
                            "le sens d'étude ("<strong>"Précédent"</strong>" ou "
                            <strong>"Suivant"</strong>"), et indiquez une ou plusieurs distances "
                            "(soit un horizon ou une fenêtre contextuelle précise) "
                            "dans le champ "<strong>"Écart(s)"</strong>" — "<code>"1"</code>
                            " étudie le mot immédiatement adjacent, "<code>"2"</code>" le mot "
                            "suivant, etc. Plusieurs distances peuvent être séparées par une "
                            "virgule, sans espace (ex : "<code>"1,2,4"</code>")."
                        </p>
                        <p>
                            "Le bouton "<strong>"Tiers ▾"</strong>" active ou désactive, pour chaque "
                            "tier, une colonne de comptage dédiée (toutes actives par défaut). "
                            "Cliquez sur "<strong>"Analyser"</strong>" pour lancer le calcul."
                        </p>
                        <p>
                            "Le tableau comporte une colonne "<strong>"Type"</strong>" (le pivot "
                            "suivi de son contexte — un "<code>"_"</code>" représente un mot ignoré "
                            "entre les deux), une colonne "<strong>"Occ. Tot."</strong>
                            " (occurrences de ce contexte sur le total d'occurrences du pivot, tous "
                            "tiers confondus), et une colonne par tier active (même logique, "
                            "restreinte à cette tier)."
                        </p>
                        <p>
                            "Le bouton "<strong>"Export Excel"</strong>" exporte le tableau tel "
                            "qu'affiché à l'instant."
                        </p>
                    </section>

                    <p class="info-footer">
                        "Version 2.0.0 — Développé pour la recherche linguistique. Pour toute "
                        "question, suggestion ou signalement de bug : "
                        "matteo.gyselinck@student.outlook.com"
                    </p>
                </div>
            </div>
        </div>
    }
}
