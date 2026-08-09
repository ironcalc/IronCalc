import ironcalc as ic
import pytest


@pytest.fixture
def um() -> ic.UserModel:
    """An empty workbook with the user API"""
    return ic.UserModel("workbook")


@pytest.fixture
def rm() -> ic.Model:
    """An empty workbook with the raw API"""
    return ic.create("workbook")
