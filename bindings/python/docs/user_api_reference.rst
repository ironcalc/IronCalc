User API Reference
------------------

This is the "user api". Models here have history, they evaluate automatically with each change and have a "diff" history.


.. method:: save_to_xlsx(file: str)

    Saves the user model to file in the XLSX format.

    ::param file: The file path to save the model to.

.. method:: save_to_icalc(file: str)

    Saves the user model to file in the internal binary ic format.

    ::param file: The file path to save the model to.

.. method:: apply_external_diffs(external_diffs: bytes)

    Applies external diffs to the model. This is used to apply changes from other instances of the model.

    ::param external_diffs: The external diffs to apply, as a byte array.

.. method:: flush_send_queue() -> bytes

    Flushes the send queue and returns the bytes to be sent to the client. This is used to send changes to the client.

.. method:: set_user_input(sheet: int, row: int, column: int, value: str)

    Sets an input in a cell, as would be done by a user typing into a spreadsheet cell.

.. method:: get_formatted_cell_value(sheet: int, row: int, column: int) -> str

    Returns the cell’s value as a formatted string, taking into account any number/currency/date formatting.

.. method:: to_bytes() -> bytes

    Returns the model as a byte array. This is useful for sending the model over a network or saving it to a file.


