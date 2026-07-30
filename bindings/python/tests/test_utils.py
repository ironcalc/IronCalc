import ironcalc as ic
import pytest


def test_column_name_from_number():
    assert ic.column_name_from_number(1) == "A"
    assert ic.column_name_from_number(26) == "Z"
    assert ic.column_name_from_number(27) == "AA"
    assert ic.column_name_from_number(16384) == "XFD"


def test_column_name_invalid():
    with pytest.raises(ic.WorkbookError):
        ic.column_name_from_number(16385)


def test_column_number_from_name():
    assert ic.column_number_from_name("A") == 1
    assert ic.column_number_from_name("XFD") == 16384
    with pytest.raises(ic.WorkbookError):
        ic.column_number_from_name("hello")


def test_quote_name():
    assert ic.quote_name("Sheet1") == "Sheet1"
    assert ic.quote_name("My Sheet") == "'My Sheet'"


def test_get_all_timezones():
    assert "UTC" in ic.get_all_timezones()
    assert "Europe/Berlin" in ic.get_all_timezones()


def test_get_supported_locales():
    assert "en" in ic.get_supported_locales()
