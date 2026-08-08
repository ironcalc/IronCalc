def test_builtin_named_styles(um):
    builtins = um.get_builtin_named_styles()
    names = [entry["name"] for entry in builtins]
    assert "Normal" in names
    assert "Good" in names
    assert "Bad" in names


def test_create_and_get_named_style(um):
    style = um.get_cell_style(0, 1, 1)
    style["font"]["b"] = True
    style["fill"]["color"] = "#DDEBF7"
    um.create_named_style("Header", style)

    assert "Header" in um.get_named_style_list()
    named = um.get_named_style("Header")
    assert named["font"]["b"] is True
    assert named["fill"]["color"] == "#DDEBF7"


def test_apply_named_style_to_selection(um):
    style = um.get_cell_style(0, 1, 1)
    style["font"]["u"] = True
    um.create_named_style("Underlined", style)

    # the selected cell must be a corner of the selected range
    um.set_selected_cell(2, 1)
    um.set_selected_range(2, 1, 2, 3)
    um.on_apply_named_style("Underlined")

    for column in range(1, 4):
        assert um.get_cell_style(0, 2, column)["font"]["u"] is True


def test_apply_builtin_named_style(um):
    um.set_selected_cell(1, 1)
    um.on_apply_named_style("Good")
    assert "Good" in um.get_named_style_list()


def test_update_named_style(um):
    style = um.get_cell_style(0, 1, 1)
    um.create_named_style("MyStyle", style)
    style["font"]["i"] = True
    um.update_named_style("MyStyle", "MyStyleRenamed", style)

    assert "MyStyleRenamed" in um.get_named_style_list()
    assert "MyStyle" not in um.get_named_style_list()
    assert um.get_named_style("MyStyleRenamed")["font"]["i"] is True


def test_delete_named_style(um):
    style = um.get_cell_style(0, 1, 1)
    um.create_named_style("Ephemeral", style)
    assert "Ephemeral" in um.get_named_style_list()
    um.delete_named_style("Ephemeral")
    assert "Ephemeral" not in um.get_named_style_list()


def test_named_style_includes(um):
    style = um.get_cell_style(0, 1, 1)
    um.create_named_style("Full", style)
    includes = um.get_named_style_includes("Full")
    # all categories are included by default
    assert all(includes.values())
