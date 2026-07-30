Top Level Methods
-----------------

This module provides a set of top-level methods for creating and loading
IronCalc models. All ``locale``, ``tz`` and ``language_id`` parameters default
to ``"en"``, ``"UTC"`` and ``"en"``.

Creating models (raw API)
^^^^^^^^^^^^^^^^^^^^^^^^^

.. function:: create(name, locale="en", tz="UTC", language_id="en") -> Model

   Creates an empty workbook with the raw API.

.. function:: load_from_xlsx(file_path, locale="en", tz="UTC", language_id="en") -> Model

   Loads a workbook from an xlsx file.

.. function:: load_from_icalc(file_name, language_id="en") -> Model

   Loads a workbook from a file in the internal binary ic format.

.. function:: load_from_bytes(bytes, language_id="en") -> Model

   Loads a workbook from bytes in the internal binary ic format (the format
   produced by ``to_bytes`` and ``save_to_icalc``).

Creating models (user API)
^^^^^^^^^^^^^^^^^^^^^^^^^^

The :class:`UserModel` constructor and static methods are the idiomatic way:

.. code-block:: python

   model = ic.UserModel("My Workbook")
   model = ic.UserModel.from_xlsx("book.xlsx")
   model = ic.UserModel.from_icalc("book.ic")
   model = ic.UserModel.from_bytes(data)

The equivalent top-level functions are also available:

.. function:: create_user_model(name, locale="en", tz="UTC", language_id="en") -> UserModel
.. function:: create_user_model_from_xlsx(file_path, locale="en", tz="UTC", language_id="en") -> UserModel
.. function:: create_user_model_from_icalc(file_name, language_id="en") -> UserModel
.. function:: create_user_model_from_bytes(bytes, language_id="en") -> UserModel

Utilities
^^^^^^^^^

.. function:: column_name_from_number(column: int) -> str

   Returns the column name ("A", "B", ..., "XFD") for a 1-based column number.

.. function:: column_number_from_name(column: str) -> int

   Returns the 1-based column number for a column name.

.. function:: quote_name(name: str) -> str

   Quotes a sheet name if needed so it can be used in a formula reference
   (``"My Sheet"`` -> ``"'My Sheet'"``).

.. function:: get_all_timezones() -> list[str]

   Returns the list of supported timezones.

.. function:: get_supported_locales() -> list[str]

   Returns the list of supported locales.
