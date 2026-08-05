"""Populates app.db with the owner's two real decks and known inventory (2026-08-02).

Text values kept in Portuguese here (deck philosophy, card notes, activity log
messages, game notes) are actual app content the owner sees on screen — this
file's own comments and docstrings are in English per the repo's convention.
"""
from db import get_app_db, init_db, log_activity

SYR_KONRAD = {
    "commander": "Syr Konrad, the Grim",
    "philosophy": "Sacrifício, recursão de cemitério, discard e gatilhos de morte de criatura. "
                  "Tema: transcender a morte e controlar o destino.",
    "cards": {
        "Swamp": 31, "Barren Moor": 1, "Mortuary Mire": 1, "Myriad Landscape": 1, "Witch's Cottage": 1,
        "Viscera Seer": 1, "Blood Artist": 1, "Zulaport Cutthroat": 1, "Gray Merchant of Asphodel": 1,
        "Grim Servant": 1, "Undead Gladiator": 1, "Ayara, First of Locthwain": 1, "Spark Reaper": 1,
        "Solemn Simulacrum": 1, "Grim Haruspex": 1, "Midnight Reaper": 1, "Doomed Necromancer": 1,
        "Nekrataal": 1, "Ravenous Chupacabra": 1, "Stinkweed Imp": 1, "Plaguecrafter": 1,
        "Accursed Marauder": 1, "Gnawing Zombie": 1, "Wight of Precinct Six": 1, "Ogre Slumlord": 1,
        "Reassembling Skeleton": 1, "Liliana's Devotee": 1, "Skirsdag High Priest": 1, "Dreadhound": 1,
        "Murderous Rider": 1,
        "Abnormal Endurance": 1, "Dark Ritual": 1, "Deadly Dispute": 1, "Death Denied": 1,
        "Demonic Gifts": 1, "Fake Your Own Death": 1, "Feed the Serpent": 1, "Ghastly Demise": 1,
        "Grasp of Darkness": 1, "Hazel's Nocturne": 1, "Heartless Act": 1, "Infernal Grasp": 1,
        "Murder": 1, "Murderous Cut": 1, "Rend Flesh": 1, "Return to Action": 1,
        "Supernatural Stamina": 1, "Tragic Slip": 1, "Village Rites": 1,
        "Deadly Tempest": 1, "Diabolic Tutor": 1, "Feed the Swarm": 1, "Final Act": 1,
        "Morality Shift": 1, "Painful Lesson": 1, "Plague Wind": 1, "Read the Bones": 1,
        "Sign in Blood": 1, "Victimize": 1, "Villainous Wrath": 1,
        "Arcane Signet": 1, "Desecrated Tomb": 1, "Elixir of Immortality": 1, "Ghoulcaller's Bell": 1,
        "Mind Stone": 1, "Sol Ring": 1, "Swiftfoot Boots": 1, "Worn Powerstone": 1,
        "Liliana, Death's Majesty": 1,
    },
    # known printings by card name (everything else has no confirmed set/lang)
    "printings": {
        "Return to Action": {"lang": "ja"},
        "Victimize": {"lang": "ja"},
    },
}

TOSHIRO = {
    "commander": "Toshiro Umezawa",
    "philosophy": "Recursão de mágicas instantâneas via cemitério (graveyard recursion mono-black).",
    "cards": {
        "Swamp": 27, "Myriad Landscape": 1, "Castle Locthwain": 1, "War Room": 1,
        "Arcane Lighthouse": 1, "Reliquary Tower": 1, "Mortuary Mire": 1, "Cabal Stronghold": 1,
        "Burnished Hart": 1, "Dread Presence": 1, "Blood Artist": 1, "Solemn Simulacrum": 1,
        "Oriq Loremage": 1, "K'rrik, Son of Yawgmoth": 1, "Gravebreaker Lamia": 1, "Sedgemoor Witch": 1,
        "Kaervek, the Punisher": 1, "Harvester of Souls": 1, "Tormod, the Desecrator": 1,
        "Crypt Ghast": 1, "Syr Konrad, the Grim": 1, "Morbid Opportunist": 1,
        "Oathkeeper, Takeno's Daisho": 1, "Arcane Signet": 1, "Wayfarer's Bauble": 1,
        "Charcoal Diamond": 1, "Dingus Staff": 1, "Sol Ring": 1, "Seance Board": 1, "Mind Stone": 1,
        "Professor Onyx": 1, "Grave Betrayal": 1, "Phyrexian Arena": 1, "Nowhere to Run": 1,
        "Black Market": 1,
        "Bile Blight": 1, "Bitter Triumph": 1, "Blood Pact": 1, "Cling to Dust": 1, "Cremate": 1,
        "Cut Down": 1, "Dark Bargain": 1, "Dark Ritual": 1, "Debt to the Kami": 1, "Dismember": 1,
        "Doom Blade": 1, "Feign Death": 1, "Go for the Throat": 1, "Grasp of Darkness": 1,
        "Heartless Act": 1, "Hero's Downfall": 1, "Infernal Grasp": 1, "Liliana's Triumph": 1,
        "Malicious Affliction": 1, "Overkill": 1, "Pharika's Libation": 1, "Price of Fame": 1,
        "Sheoldred's Edict": 1, "Succumb to Temptation": 1, "Sudden Death": 1, "Sudden Spoiling": 1,
        "Tendrils of Corruption": 1, "Terror": 1, "Thrilling Encore": 1, "Tragic Slip": 1,
        "Undying Evil": 1, "Vona's Hunger": 1, "Withering Boon": 1, "Withering Torment": 1,
        "Diabolic Tutor": 1, "Exsanguinate": 1, "Sign in Blood": 1, "Rise of the Dark Realms": 1,
    },
    "printings": {},
}

# Cards not currently in any deck, but owned (see Colecao - Inventario Geral.md in the vault)
FREE_CARDS = [
    "Cornered by Black Mages", "Poison the Cup", "Tribute to Hunger", "Undying Malice",
    "Sudden Edict", "Indulgent Tormentor", "Lord of the Forsaken", "Morlun, Devourer of Spiders",
    "Kiku, Night's Flower", "Ashnod's Intervention", "Black Cat, Cunning Thief", "Butcher Ghoul",
    "Crippling Fear", "Dusk Legion Zealot", "Dutiful Return", "Gonti, Lord of Luxury",
    "Liliana's Reaver", "March of the Returned", "Massacre Girl", "Merciless Resolve",
    "Retrofitted Transmogrant", "Toshiro Umezawa", "Altar's Reap",
]


def seed_deck(con, deck_def):
    cur = con.execute(
        "INSERT INTO decks (name, commander_name, philosophy) VALUES (?, ?, ?)",
        (deck_def["commander"], deck_def["commander"], deck_def["philosophy"]),
    )
    deck_id = cur.lastrowid
    for card_name, qty in deck_def["cards"].items():
        is_commander = 1 if card_name == deck_def["commander"] and card_name not in (
            "Syr Konrad, the Grim", "Toshiro Umezawa") else 0
        con.execute(
            "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander) VALUES (?, ?, ?, ?)",
            (deck_id, card_name, qty, 0),
        )
    # commander as its own entry (wasn't in the card dicts above)
    con.execute(
        "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander) VALUES (?, ?, 1, 1)",
        (deck_id, deck_def["commander"]),
    )
    # collection: each deck card becomes one allocated copy
    for card_name, qty in deck_def["cards"].items():
        printing = deck_def["printings"].get(card_name, {})
        con.execute(
            "INSERT INTO collection (card_name, set_code, lang, quantity, allocated_deck_id) "
            "VALUES (?, ?, ?, ?, ?)",
            (card_name, printing.get("set"), printing.get("lang", "en"), qty, deck_id),
        )
    con.execute(
        "INSERT INTO collection (card_name, set_code, lang, quantity, allocated_deck_id) VALUES (?, NULL, 'en', 1, ?)",
        (deck_def["commander"], deck_id),
    )
    total = sum(deck_def["cards"].values()) + 1
    log_activity(con, "deck_built", f"Deck {deck_def['commander']} montado ({total} cartas)")
    return deck_id


def main():
    init_db()
    con = get_app_db()
    con.execute("DELETE FROM game_highlights")
    con.execute("DELETE FROM games")
    con.execute("DELETE FROM activity")
    con.execute("DELETE FROM collection")
    con.execute("DELETE FROM deck_cards")
    con.execute("DELETE FROM decks")

    syr_id = seed_deck(con, SYR_KONRAD)
    tosh_id = seed_deck(con, TOSHIRO)

    for card_name in FREE_CARDS:
        con.execute(
            "INSERT INTO collection (card_name, set_code, lang, quantity, allocated_deck_id, notes) "
            "VALUES (?, NULL, 'en', 1, NULL, 'Livre — fora de deck montado no momento')",
            (card_name,),
        )

    # a few sample games, to show the stats screen working with real reported data
    games = [
        (syr_id, "2026-07-20", "vitoria", "Golos, Zaxara, Meren", 9, "Combo Mindcrank+Bloodchief não saiu, venceu no attrition"),
        (syr_id, "2026-07-27", "derrota", "Atraxa, Korvold", 7, "Board wipe adversário levou tudo antes do Konrad estabilizar"),
        (tosh_id, "2026-07-13", "vitoria", "Muldrotha, Karador", 8, "Phyrexian Arena + Harvester of Souls gerou vantagem de cartas enorme"),
    ]
    for deck_id, date, result, opps, turns, notes in games:
        cur = con.execute(
            "INSERT INTO games (deck_id, played_at, result, opponents, turns, notes) VALUES (?,?,?,?,?,?)",
            (deck_id, date, result, opps, turns, notes),
        )
    con.commit()

    n_decks = con.execute("SELECT COUNT(*) FROM decks").fetchone()[0]
    n_cards = con.execute("SELECT SUM(quantity) FROM deck_cards").fetchone()[0]
    n_coll = con.execute("SELECT COUNT(*) FROM collection").fetchone()[0]
    print(f"Decks: {n_decks} | cartas em decks: {n_cards} | linhas de coleção: {n_coll}")
    con.close()


if __name__ == "__main__":
    main()
