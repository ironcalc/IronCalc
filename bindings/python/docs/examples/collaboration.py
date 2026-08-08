"""Keep two models in sync exchanging diffs (the collaboration mechanism)."""

import ironcalc as ic

alice = ic.UserModel("shared")
bob = ic.UserModel("shared")

# Alice makes some edits
alice.set_user_input(0, 1, 1, "100")
alice.set_user_input(0, 2, 1, "=A1*2")
alice.update_range_style(0, 1, 1, 2, 1, "font.b", "true")

# ... and sends the resulting diffs to Bob
diffs = alice.flush_send_queue()
bob.apply_external_diffs(diffs)

assert bob.get_formatted_cell_value(0, 2, 1) == "200"
assert bob.get_cell_style(0, 1, 1)["font"]["b"] is True

# Edits flow in both directions
bob.set_user_input(0, 3, 1, "=SUM(A1:A2)")
alice.apply_external_diffs(bob.flush_send_queue())

assert alice.get_formatted_cell_value(0, 3, 1) == "300"
print("both models in sync")
