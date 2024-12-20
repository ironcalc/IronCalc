---
layout: doc
outline: deep
lang: en-US
---

# Optional Arguments

::: warning
**Note:** This draft page is under construction 🚧
:::

Any IronCalc function may accept zero, one or more arguments, which are values passed to the function when it is called from a spreadsheet formula.

Many function arguments are _required_. For such arguments, always pass a suitable value in the function call.

Some function arguments are _optional_. Optional arguments need not be passed in the function call and, in such cases, the function instead uses a predefined default value.

Consider a notional function called _FN\_NAME_, with the following syntax:

<p style="font-weight:bold;text-align:center;">FN_NAME(<span title="Number" style="color:#1E88E5">arg1</span>, <span title="Number" style="color:#1E88E5">arg2</span>, <span title="Number" style="color:#1E88E5">arg3</span>, <span title="Number" style="color:#1E88E5">arg4</span>=def1, <span title="Number" style="color:#1E88E5">arg5</span>=def2,  <span title="Number" style="color:#1E88E5">arg6</span>=def3) => <span title="Number" style="color:#1E88E5" >fn_name</span></p>

Notes about this syntax:

* _FN_NAME_ is a function that takes six arguments (_arg1_, _arg2_, _arg3_, _arg4_, _arg5_ and _arg6_) and returns a value referred to as _fn_name_.
* For convenience in this case, all arguments and the returned value are colour-coded to indicate that they are numbers.
* Arguments _arg1_, _arg2_ and _arg3_ are <u>required</u> arguments and this would normally be stated in the **Argument descriptions** section of the function's description page.
* Arguments _arg4_, _arg5_ and _arg6_ are <u>optional</u> arguments and again this would normally be stated in the **Argument descriptions** section of the function's description page. In addition, optional arguments are usually indicated by the specification of a default value in the syntax. 
   * If _arg4_ is omitted, then the value _def1_ is assumed.
   * If _arg5_ is omitted, then the value _def2_ is assumed. 
   * If _arg6_ is omitted, then the value _def3_ is assumed.

With this syntax, the following would all be valid calls to the _FN_NAME_ function:

**=FN\_NAME(1,2,3)**. All optional arguments omitted.

**=FN\_NAME(1,2,3,4)**. _arg4_ set to 4; optional arguments _arg5_ and _arg6_ assume default values.

**=FN\_NAME(1,2,3,,5)**.  _arg5_ set to 5; optional arguments _arg4_ and _arg6_ assume  default values.

**=FN\_NAME(1,2,3,,,6)**. _arg6_ set to 6; optional arguments _arg4_ and _arg5_ assume  default values.

**=FN\_NAME(1,2,3,4,5)**. _arg4_ and  _arg5_ set to 4 and 5 respectively; optional argument _arg6_ assumes default value.

**=FN\_NAME(1,2,3,4,,6)**. _arg4_ and _arg6_ set to 4 and 6 respectively; optional argument _arg5_ assumes default value.

**=FN\_NAME(1,2,3,,5,6)**. _arg5_ and _arg6_ set to 5 and 6 respectively; optional argument _arg4_ assumes default value.

**=FN\_NAME(1,2,3,4,5,6)**. Values passed for all optional arguments.
