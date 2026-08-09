import ironcalc as ic


def test_xlsx_roundtrip_user_model(um, tmp_path):
    um.set_user_input(0, 1, 1, "=6*7")
    um.update_range_style(0, 1, 1, 1, 1, "font.b", "true")
    file = str(tmp_path / "book.xlsx")
    um.save_to_xlsx(file)

    model = ic.UserModel.from_xlsx(file)
    assert model.get_formatted_cell_value(0, 1, 1) == "42"
    assert model.get_cell_style(0, 1, 1)["font"]["b"] is True


def test_xlsx_roundtrip_raw_model(rm, tmp_path):
    rm.set_user_input(0, 1, 1, "=6*7")
    rm.evaluate()
    file = str(tmp_path / "book.xlsx")
    rm.save_to_xlsx(file)

    model = ic.load_from_xlsx(file)
    model.evaluate()
    assert model.get_formatted_cell_value(0, 1, 1) == "42"


def test_icalc_roundtrip(um, tmp_path):
    um.set_user_input(0, 1, 1, "hello")
    file = str(tmp_path / "book.ic")
    um.save_to_icalc(file)

    model = ic.UserModel.from_icalc(file)
    assert model.get_formatted_cell_value(0, 1, 1) == "hello"


def test_bytes_roundtrip_both_apis(um):
    um.set_user_input(0, 1, 1, "=1+2")
    data = bytes(um.to_bytes())

    user = ic.UserModel.from_bytes(data)
    assert user.get_formatted_cell_value(0, 1, 1) == "3"

    raw = ic.load_from_bytes(data)
    assert raw.get_formatted_cell_value(0, 1, 1) == "3"


def test_workbook_name(um):
    assert um.get_name() == "workbook"
    um.set_name("renamed")
    assert um.get_name() == "renamed"


def test_locale_timezone_language(um):
    assert um.get_timezone() == "UTC"
    assert um.get_locale() == "en"
    assert um.get_language() == "en"
    um.set_timezone("Europe/Berlin")
    assert um.get_timezone() == "Europe/Berlin"


def test_fmt_settings(um):
    settings = um.get_fmt_settings()
    assert "currency" in settings
    assert "short_date" in settings


def test_version():
    assert ic.__version__
