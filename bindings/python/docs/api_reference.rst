
API Reference
-------------

In general methods in IronCalc use a 0-index base for the the sheet index and 1-index base for the row and column indexes.


.. method:: evaluate()

   Evaluates the model. This needs to be done after each change, otherwise the model might be on a broken state.

.. method:: set_user_input(sheet: int, row: int, column: int, value: str)

      Sets an input in a cell, as would be done by a user typing into a spreadsheet cell.

      :param sheet: The sheet index (0-based).
      :param row: The 1-based row index (first row is 1).
      :param column: The 1-based column index (column “A” is 1).
      :param value: The value to set, e.g. ``"123"`` or ``"=A1*2"``.

.. method:: clear_cell_contents(sheet: int, row: int, column: int)

      Removes the content of the cell but leaves the style intact.

      :param sheet: The sheet index (0-based).
      :param row: The 1-based row index (first row is 1).
      :param column: The 1-based column index (column “A” is 1).

.. method:: get_cell_content(sheet: int, row: int, column: int) -> str

      Returns the raw content of a cell. If the cell contains a formula, 
      the returned string starts with ``"="``.

      :param sheet: The sheet index (0-based).
      :param row: The 1-based row index.
      :param column: The 1-based column index.
      :returns: The raw content, or an empty string if the cell is empty.

.. method:: get_cell_type(sheet: int, row: int, column: int) -> PyCellType

      Returns the type of the cell (number, boolean, string, error, etc.).

      :param sheet: The sheet index (0-based).
      :param row: The 1-based row index.
      :param column: The 1-based column index.
      :rtype: PyCellType

.. method:: get_formatted_cell_value(sheet: int, row: int, column: int) -> str

      Returns the cell’s value as a formatted string, taking into 
      account any number/currency/date formatting.

      :param sheet: The sheet index (0-based).
      :param row: The 1-based row index.
      :param column: The 1-based column index.
      :returns: Formatted string of the cell’s value.

.. method:: set_cell_style(sheet: int, row: int, column: int, style: PyStyle)

      Sets the style of the cell at (sheet, row, column).

      :param sheet: The sheet index (0-based).
      :param row: The 1-based row index.
      :param column: The 1-based column index.
      :param style: A PyStyle object specifying the style.

.. method:: get_cell_style(sheet: int, row: int, column: int) -> PyStyle

      Retrieves the style of the specified cell.

      :param sheet: The sheet index (0-based).
      :param row: The 1-based row index.
      :param column: The 1-based column index.
      :returns: A PyStyle object describing the cell’s style.

.. method:: insert_rows(sheet: int, row: int, row_count: int)

      Inserts new rows.

      :param sheet: The sheet index (0-based).
      :param row: The position before which new rows are inserted (1-based).
      :param row_count: The number of rows to insert.

.. method:: insert_columns(sheet: int, column: int, column_count: int)

      Inserts new columns.

      :param sheet: The sheet index (0-based).
      :param column: The position before which new columns are inserted (1-based).
      :param column_count: The number of columns to insert.

.. method:: delete_rows(sheet: int, row: int, row_count: int)

      Deletes a range of rows.

      :param sheet: The sheet index (0-based).
      :param row: The starting row to delete (1-based).
      :param row_count: How many rows to delete.

.. method:: delete_columns(sheet: int, column: int, column_count: int)

      Deletes a range of columns.

      :param sheet: The sheet index (0-based).
      :param column: The starting column to delete (1-based).
      :param column_count: How many columns to delete.

.. method:: get_column_width(sheet: int, column: int) -> float

      Retrieves the width of a given column.

      :param sheet: The sheet index (0-based).
      :param column: The 1-based column index.
      :rtype: float

.. method:: get_row_height(sheet: int, row: int) -> float

      Retrieves the height of a given row.

      :param sheet: The sheet index (0-based).
      :param row: The 1-based row index.
      :rtype: float

.. method:: set_column_width(sheet: int, column: int, width: float)

      Sets the width of a given column.

      :param sheet: The sheet index (0-based).
      :param column: The 1-based column index.
      :param width: The desired width (float).

.. method:: set_row_height(sheet: int, row: int, height: float)

      Sets the height of a given row.

      :param sheet: The sheet index (0-based).
      :param row: The 1-based row index.
      :param height: The desired height (float).

.. method:: get_frozen_columns_count(sheet: int) -> int

      Returns the number of columns frozen (pinned) on the left side of the sheet.

      :param sheet: The sheet index (0-based).
      :rtype: int

.. method:: get_frozen_rows_count(sheet: int) -> int

      Returns the number of rows frozen (pinned) at the top of the sheet.

      :param sheet: The sheet index (0-based).
      :rtype: int

.. method:: set_frozen_columns_count(sheet: int, column_count: int)

      Sets how many columns are frozen (pinned) on the left.

      :param sheet: The sheet index (0-based).
      :param column_count: The number of frozen columns (0-based).

.. method:: set_frozen_rows_count(sheet: int, row_count: int)

      Sets how many rows are frozen (pinned) at the top.

      :param sheet: The sheet index (0-based).
      :param row_count: The number of frozen rows (0-based).

.. method:: get_worksheets_properties() -> List[PySheetProperty]

      Returns a list of :class:`PySheetProperty` describing each worksheet’s 
      name, visibility state, ID, and tab color.

      :rtype: list of PySheetProperty

.. method:: set_sheet_color(sheet: int, color: str)

      Sets the tab color of a sheet. Use an empty string to clear the color.

      :param sheet: The sheet index (0-based).
      :param color: A color in “#RRGGBB” format, or empty to remove color.

.. method:: add_sheet(sheet_name: str)

      Creates a new sheet with the specified name.

      :param sheet_name: The name to give the new sheet.

.. method:: new_sheet()

      Creates a new sheet with an auto-generated name.

.. method:: delete_sheet(sheet: int)

      Deletes the sheet at the given index.

      :param sheet: The sheet index (0-based).

.. method:: rename_sheet(sheet: int, new_name: str)

      Renames the sheet at the given index.

      :param sheet: The sheet index (0-based).
      :param new_name: The new sheet name.

.. method:: test_panic()

      A test method that deliberately panics in Rust. 
      Used for testing panic handling at the method level.

      :raises WorkbookError: (wrapped Rust panic)
