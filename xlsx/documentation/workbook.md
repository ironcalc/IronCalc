workbook.xlm: worksheets, define names and relationships
========================================================

The most important thing we will find in `workbook.xml` is a list of sheets and a list of defined names


For example the list of sheets might be something like:

```xml
<sheets>
    <sheet name="Sheet1" sheetId="1" r:id="rId1"/>
    <sheet name="Chart1" sheetId="6" r:id="rId2"/>
    <sheet name="Second" sheetId="3" r:id="rId3"/>
    <sheet name="Sheet4" sheetId="8" r:id="rId4"/>
    <sheet name="shared" sheetId="9" r:id="rId5"/>
    <sheet name="Table" sheetId="7" r:id="rId6"/>
    <sheet name="Sheet2" sheetId="2" r:id="rId7"/>
    <sheet name="Created fourth" sheetId="4" r:id="rId8"/>
    <sheet name="Hidden" sheetId="5" state="hidden" r:id="rId9"/>
</sheets>
```

The order is the order they will appear in the workbook. `sheetId` identifies the sheet and does not change if we reorder the sheets.

This example has three defined names. Those that have a `localSheetId` attribute are scoped to a sheet. Note that the `localSheetId` refers to the order in the sheet list (0-indexed) and not the `sheetId`.

A sheet can hve one of three states:

* visible
* hidden
* very hidden

To understand what file belongs to each sheet we have to do a bit of work. we will also understand the sheet "`Chart1`" is not a spreadsheet that we what to import but a "chart" sheet.
This is where the relationships file comes in (xl/_rels/workbook.xml.rels). In our case it is something like:

```xml
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId8" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet7.xml"/>
<Relationship Id="rId13" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>
<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet2.xml"/>
<Relationship Id="rId7" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet6.xml"/>
<Relationship Id="rId12" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/chartsheet" Target="chartsheets/sheet1.xml"/>
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
<Relationship Id="rId6" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet5.xml"/>
<Relationship Id="rId11" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/>
<Relationship Id="rId5" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet4.xml"/>
<Relationship Id="rId10" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotCacheDefinition" Target="pivotCache/pivotCacheDefinition1.xml"/>
<Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet3.xml"/>
<Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet8.xml"/>
<Relationship Id="rId14" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/calcChain" Target="calcChain.xml"/>
</Relationships>
```

The `r:id` attribute in the sheet list links the sheet to this relationships file. For instance the sheet "shared" has an relationships id "rIdr5" that links to the file "`worksheets/sheet4.xml`" that is of type "worksheet".
Note that the second sheet "Chart" has id `rId2` that links to the file "`chartsheets/sheet1.xml`" and is of type "chartsheet". In IronCalc we ignore those sheets.

```xml
<definedNames>
    <definedName name="answer" localSheetId="4">shared!$G$5</definedName>
    <definedName name="answer2" localSheetId="0">Sheet1!$I$6</definedName>
    <definedName name="local_thing" localSheetId="2">Second!$B$1:$B$9</definedName>
    <definedName name="numbers">Sheet1!$A$16:$A$18</definedName>
    <definedName name="quantum">Sheet1!$C$14</definedName>
</definedNames>
```

So `answer2` is scoped to `Sheet1` and `answer` is scoped to `shared`.