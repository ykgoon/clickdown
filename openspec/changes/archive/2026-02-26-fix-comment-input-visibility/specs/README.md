## No New Specifications

This change is a **bug fix** to the existing comment input form visibility. It does not introduce new capabilities or modify existing specification-level requirements.

The fix addresses an implementation detail (when the input form becomes visible) without changing:
- The comment creation API contract
- The user-visible behavior after the form appears
- The validation, saving, or cancellation logic
- Any external interfaces or integrations

All behavior remains as specified in the existing comment specifications. The only change is the **timing** of when the input form renders, which is an internal UI implementation detail.
