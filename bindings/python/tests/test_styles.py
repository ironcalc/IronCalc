def test_update_range_style_bold(um):
    um.set_user_input(0, 1, 1, "Title")
    um.update_range_style(0, 1, 1, 1, 1, "font.b", "true")
    style = um.get_cell_style(0, 1, 1)
    assert style["font"]["b"] is True


def test_update_range_style_applies_to_whole_range(um):
    um.update_range_style(0, 1, 1, 3, 2, "fill.color", "#FFFF00")
    for row in range(1, 4):
        for column in range(1, 3):
            style = um.get_cell_style(0, row, column)
            assert style["fill"]["color"] == "#FFFF00"
    # outside the range nothing changed
    assert um.get_cell_style(0, 4, 1)["fill"].get("color") != "#FFFF00"


def test_update_range_style_alignment(um):
    um.update_range_style(0, 1, 1, 1, 1, "alignment.horizontal", "center")
    style = um.get_cell_style(0, 1, 1)
    assert style["alignment"]["horizontal"] == "center"


def test_update_range_style_number_format(um):
    um.set_user_input(0, 1, 1, "0.12345")
    um.update_range_style(0, 1, 1, 1, 1, "num_fmt", "0.00%")
    assert um.get_formatted_cell_value(0, 1, 1) == "12.35%"


def test_range_clear_formatting(um):
    um.set_user_input(0, 1, 1, "data")
    um.update_range_style(0, 1, 1, 1, 1, "font.i", "true")
    um.range_clear_formatting(0, 1, 1, 1, 1)
    style = um.get_cell_style(0, 1, 1)
    # false booleans are omitted from the style dictionary
    assert style["font"].get("i") is not True
    assert um.get_formatted_cell_value(0, 1, 1) == "data"


def test_set_area_with_border(um):
    border_area = {"item": {"style": "thin", "color": "#FF0000"}, "type": "All"}
    um.set_area_with_border(0, 1, 1, 2, 2, border_area)
    style = um.get_cell_style(0, 1, 1)
    assert style["border"]["top"] == {"style": "thin", "color": "#FF0000"}
    assert style["border"]["left"] == {"style": "thin", "color": "#FF0000"}


def test_raw_set_cell_style_roundtrip(rm):
    style = rm.get_cell_style(0, 1, 1)
    style["font"]["b"] = True
    style["fill"]["color"] = "#00FF00"
    style["num_fmt"] = "#,##0.00"
    rm.set_cell_style(0, 1, 1, style)

    new_style = rm.get_cell_style(0, 1, 1)
    assert new_style["font"]["b"] is True
    assert new_style["fill"]["color"] == "#00FF00"
    assert new_style["num_fmt"] == "#,##0.00"


def test_raw_column_and_row_styles(rm):
    style = rm.get_cell_style(0, 1, 1)
    style["font"]["i"] = True
    rm.set_column_style(0, 3, style)
    rm.set_row_style(0, 5, style)

    assert rm.get_column_style(0, 3)["font"]["i"] is True
    assert rm.get_row_style(0, 5)["font"]["i"] is True
    assert rm.get_column_style(0, 4) is None

    rm.delete_column_style(0, 3)
    assert rm.get_column_style(0, 3) is None
    rm.delete_row_style(0, 5)
    # the row entry survives with a default (non-italic) style
    row_style = rm.get_row_style(0, 5)
    assert row_style is None or row_style["font"].get("i") is not True


def test_get_extended_cell_style(um):
    # the extended style overlays conditional formatting decorations
    um.add_conditional_formatting(
        0,
        "A1:A5",
        {
            "type": "ColorScale",
            "thresholds": [
                {"cfvo": "Min", "color": "#FF0000"},
                {"cfvo": "Max", "color": "#00FF00"},
            ],
        },
    )
    um.set_user_input(0, 1, 1, "1")
    um.set_user_input(0, 2, 1, "10")
    ext = um.get_extended_cell_style(0, 1, 1)
    assert ext["style"]["fill"]["color"] is not None
    assert ext["icon"] is None
