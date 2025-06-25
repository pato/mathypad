# Todo 

i want you to make sure that we cannot add currency rates unless it is the same currency, then i want you to critically review all the currency operators to make sure we do not accidentally allow operations across diferent currencies

it looks like we have duplicate logic:  The main function evaluate_expression_with_context calls evaluate_tokens_stream_with_context,
  not evaluate_tokens_with_units_and_context. Let me check what evaluate_tokens_stream_with_context does.

- [ ] Add "12 as % of 60" support
- [ ] Add support for comments after expressions
- [ ] find a way to get better precision (maybe a year should just be 365 days? a month only 30?)
- [x] fix syntax bug with "line1/month * 3"
- [x] i want you to add support for "quarter" as a unit of time, it is equivalent to 3 months.
- [x] Doesn't handle currency (at the very least preserving unit, not including currency conversion)
- [x] Update the readme.md with all the latest functionality
- [x] Add support for sqrt()
- [x] Update the website with all the latest functionality
- [x] Add support for exponents
- [x] Add changelog support inside the app, so that users can see what was added since they last ran it (if there was any update)
- [x] Doesn't seem to handle days or weeks or months or years
- [x] Add UI snapshot tests
