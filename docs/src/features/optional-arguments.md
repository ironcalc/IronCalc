---
layout: doc
outline: deep
lang: en-US
---

# Error Types

::: warning
**Note:** This page is in construction 🚧
:::


Some functions have optional arguments. For instance:

XLOOKUP(lookup_value, lookup_array, return_array, if_not_found="#N/A", match_mode=0, search_mode=1)

The three first arguments are mandatory and the last three are optional.
Optional argumenst must have a default value.

In this example if you don't want to specify the if_not_found and match_mode arguments you can leave them out:

XLOOKUP(lookup_value, lookup_array, return_array, , , -2)

That would use the default arguments for if_not_found and match_mode arguments.

