# Task verification evidence

Record date, branch/source commit, environment and exact commands. One entry per test or explicitly enumerated parameterized family:

| Field | Evidence |
|---|---|
| Test | Stable identifier and command |
| Null hypothesis | Concrete failure claim this check could expose |
| Testing for | Behavior, inputs and independently derived oracle |
| Expected | Observable result specified before execution |
| Actual | Observed result, or not run / blocked with reason |
| Issue response | Correction and rerun plan; if passing, action on future failure |

Record negative controls/mutations separately: deliberate fault, expected failing test, actual failure, restoration, and successful rerun. State coverage limits. Do not infer integration success from unit tests or CI success from the presence of a workflow file.
