COLOR_SCALE = {
    "type": "ColorScale",
    "thresholds": [
        {"cfvo": "Min", "color": "#FF0000"},
        {"cfvo": "Max", "color": "#00FF00"},
    ],
}

CELL_IS_GREATER_THAN_5 = {
    "type": "CellIs",
    "operator": "GreaterThan",
    "formula": "5",
    "formula2": None,
    "format": {"fill": {"color": "#FFC7CE"}},
    "stop_if_true": False,
}


def test_empty_list_initially(um):
    assert um.get_conditional_formatting_list(0) == []


def test_add_and_retrieve(um):
    um.add_conditional_formatting(0, "A1:A5", COLOR_SCALE)
    rules = um.get_conditional_formatting_list(0)
    assert len(rules) == 1
    assert rules[0]["range"] == "A1:A5"
    assert rules[0]["cf_rule"]["type"] == "ColorScale"


def test_cell_is_rule(um):
    um.add_conditional_formatting(0, "B1:B10", CELL_IS_GREATER_THAN_5)
    rules = um.get_conditional_formatting_list(0)
    rule = rules[0]["cf_rule"]
    assert rule["operator"] == "GreaterThan"
    assert rule["formula"] == "5"

    dxf = um.get_dxf_for_conditional_formatting(0, 0)
    assert dxf["fill"]["color"] == "#FFC7CE"


def test_priority_order(um):
    um.add_conditional_formatting(0, "A1:A5", COLOR_SCALE)
    um.add_conditional_formatting(0, "B1:B5", CELL_IS_GREATER_THAN_5)
    rules = um.get_conditional_formatting_list(0)
    # sorted by priority descending: the last added rule comes first
    assert rules[0]["range"] == "B1:B5"
    assert rules[1]["range"] == "A1:A5"

    um.raise_conditional_formatting_priority(0, 0)
    rules = um.get_conditional_formatting_list(0)
    assert rules[0]["range"] == "A1:A5"

    um.lower_conditional_formatting_priority(0, 0)
    rules = um.get_conditional_formatting_list(0)
    assert rules[0]["range"] == "B1:B5"


def test_update_rule(um):
    um.add_conditional_formatting(0, "A1:A5", COLOR_SCALE)
    um.update_conditional_formatting(0, 0, "A1:A20", CELL_IS_GREATER_THAN_5)
    rules = um.get_conditional_formatting_list(0)
    assert len(rules) == 1
    assert rules[0]["range"] == "A1:A20"
    assert rules[0]["cf_rule"]["type"] == "CellIs"


def test_delete_rule(um):
    um.add_conditional_formatting(0, "A1:A5", COLOR_SCALE)
    um.delete_conditional_formatting(0, 0)
    assert um.get_conditional_formatting_list(0) == []


def test_cf_survives_xlsx_roundtrip(um, tmp_path):
    import ironcalc as ic

    um.add_conditional_formatting(0, "A1:A5", CELL_IS_GREATER_THAN_5)
    file = str(tmp_path / "cf.xlsx")
    um.save_to_xlsx(file)

    model = ic.UserModel.from_xlsx(file)
    rules = model.get_conditional_formatting_list(0)
    assert len(rules) == 1
    assert rules[0]["cf_rule"]["type"] == "CellIs"
