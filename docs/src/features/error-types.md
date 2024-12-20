---
layout: doc
outline: deep
lang: en-US
---

# Error Types

::: warning
**Note:** This page is in construction 🚧
:::

When working with formulas, you may encounter these common errors:

---

### **`#ERROR!`**

**Cause:** General formula issue, like syntax errors or invalid references.  
**Fix:** Check the formula for mistakes or invalid cell references.

---

### **`#VALUE!`**

**Cause:** Mismatched data types (e.g., text used where numbers are expected).  
**Fix:** Ensure input types are correct; convert text to numbers if needed.

---

### **`#DIV/0!`**

**Cause:** Division by zero or an empty cell.  
**Fix:** Ensure the denominator isn’t zero or blank. Use `IF` to handle such cases:

```
=IF(B1=0, "N/A", A1/B1)
```

### **`#NAME?`**

**Cause:** Unrecognized text in the formula (e.g., misspelled function names or undefined named ranges).  
**Fix:** Correct spelling or define the missing name.

### **`#REF!`**

**Cause:** Invalid cell reference, often from deleting cells used in a formula.  
**Fix:** Update the formula with correct references.

### **`#NUM!`**

**Cause:** Invalid numeric operation (e.g., calculating a square root of a negative number).  
**Fix:** Adjust the formula to ensure valid numeric operations.

### **`#N/A`**

**Cause:** A value is not available, often in lookup functions like VLOOKUP.  
**Fix:** Ensure the lookup value exists or use IFNA() to handle missing values:

```
=IFNA(VLOOKUP(A1, B1:C10, 2, FALSE), "Not Found")
```

### **`#NULL!`**

**Cause:** Incorrect range operator in a formula (e.g., missing a colon between cell references).  
**Fix:** Use correct range operators (e.g., A1:A10).


### **`#CIRC!`**

**Cause:** Circular reference.  
**Fix:** Remove the circular reference.


### **`#####`**

**Cause:** The column isn’t wide enough to display the value.  
**Fix:** Resize the column width to fit the content.
