
Usage Examples
--------------

Creating an Empty Model
^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: python

   import ironcalc as ic

   model = ic.create("My Workbook", "en", "UTC")

Loading from XLSX
^^^^^^^^^^^^^^^^^

.. code-block:: python

   import ironcalc as ic

   model = ic.load_from_xlsx("example.xlsx", "en", "UTC")

Modifying and Saving
^^^^^^^^^^^^^^^^^^^^

.. code-block:: python

   model = ic.create("model", "en", "UTC")
   model.set_user_input(0, 1, 1, "123")
   model.set_user_input(0, 1, 2, "=A1*2")
   model.evaluate()

   # Save to XLSX
   model.save_to_xlsx("updated.xlsx")

   # Or save to the binary format
   model.save_to_icalc("my_workbook.icalc")
