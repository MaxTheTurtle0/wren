# UI hill climb

A loop of independent design reviews and improvements to Wren's interface. Each round captures every screen in light and dark mode from the demo (`capture.sh`). Two graders score them independently with the same rubric, and their combined findings drive the next round of changes.

- **Grader A:** Claude Opus 5.5 (effort high), acting as a senior UI/UX designer.
- **Grader B:** Codex gpt-6.1-sol (reasoning effort high), grading independently.
- **Independence:** fresh graders every round. Neither sees the other's scores, earlier rounds or the code.
- **Capture:** `docs/ui-hill-climb/capture.sh round-N [output-root]` (release build, Xvfb, software GL). 36 screens per mode.
- **Screenshots:** rounds 0-8 are committed under `round-N/`. Rounds 9 onward were captured outside the repository to keep it small; only the final round, `round-16/`, is committed.

There were two runs:

1. **Rounds 0-8:** the rubric scored hierarchy, spacing, typography, consistency, contrast and accessibility, feedback, states and navigation.
2. **Rounds 9 onward:** aimed at 9.0 from both graders, with a focus on contrast, typography, colour, spacing and visual polish. The rubric scores contrast, typography, colour, spacing and hierarchy per dimension, and each grader names what blocks a 9.

## Overall scores

| Round | Opus 5.5 | Codex gpt-6.1-sol | Mean |
| --- | ---: | ---: | ---: |
| 0 | 6.0 | 6.5 | 6.25 |
| 1 | 7.0 | 7.0 | 7.00 |
| 2 | 7.0 | 7.5 | 7.25 |
| 3 | 7.5 | 7.8 | 7.65 |
| 4 | 7.5 | 8.0 | 7.75 |
| 5 | 7.5 | 8.0 | 7.75 |
| 6 | 8.0 | 8.0 | 8.00 |
| 7 | 8.0 | 8.0 | 8.00 |
| 8 | 8.0 | 8.0 | 8.00 |
| 9 | 8.0 | 8.0 | 8.00 |
| 10 | 8.0 | 7.5 | 7.75 |
| 11 | 8.0 | 8.0 | 8.00 |
| 12 | 8.0 | 8.0 | 8.00 |
| 13 | 8.0 | 8.0 | 8.00 |
| 14 | 8.0 | 8.0 | 8.00 |
| 15 | 8.0 | 8.0 | 8.00 |
| 16 | 8.0 | 8.0 | 8.00 |

## Dimension scores, rounds 9 onward

Each cell is Opus / Codex.

| Round | Contrast | Typography | Colour | Spacing | Hierarchy |
| --- | --- | --- | --- | --- | --- |
| 9 | 8.5 / 9 | 8 / 8 | 8 / 8.5 | 7.5 / 8 | 8 / 8 |
| 10 | 9 / 7.5 | 8 / 8 | 8 / 8.5 | 7.5 / 7.5 | 8 / 7.5 |
| 11 | 9 / 8.5 | 8 / 8 | 8 / 8.5 | 7.5 / 7.5 | 8 / 7.5 |
| 12 | 9 / 8.5 | 8 / 8 | 8 / 8.5 | 7.5 / 8 | 8 / 8 |
| 13 | 8.5 / 9 | 8 / 8 | 7.5 / 8.5 | 7.5 / 8 | 8 / 8 |
| 14 | 8 / 8 | 8 / 8 | 8 / 8.5 | 7.5 / 8 | 8.5 / 8 |
| 15 | 9 / 8.5 | 8 / 8 | 8 / 8.5 | 7.5 / 7.5 | 8 / 8 |
| 16 | 8.5 / 8.5 | 8 / 8 | 7.5 / 8.5 | 8 / 7.5 | 8 / 8 |

## Per-screen scores

Each cell is Opus / Codex.

| Screen | R0 | R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 | R9 | R10 | R11 | R12 | R13 | R14 | R15 | R16 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| chats | 6.5 / 7 | 7 / 7 | 8 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8.5 / 8 | 8.5 / 8 | 8.5 / 8 | 8.5 / 8 | 8.5 / 8 | 8.5 / 8 | 8 / 8 | 8.5 / 8.5 | 8.5 / 8 | 8.5 / 8 | 8 / 8 |
| chats-wide | 6 / 6.5 | 6.5 / 6.5 | 7 / 7.5 | 7.5 / 7.5 | 7.5 / 8 | 7 / 7.5 | 8 / 8 | 7.5 / 8 | 8.5 / 8 | 8 / 8 | 8 / 8 | 8.5 / 8 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8 | 8.5 / 8.5 |
| chats-narrow | 5 / 5.5 | 6 / 6 | 7.5 / 7 | 7 / 7 | 7.5 / 7.5 | 7.5 / 7.5 | 7.5 / 7.5 | 7.5 / 7 | 7 / 7.5 | 7.5 / 7.5 | 7.5 / 7 | 7.5 / 7.5 | 7.5 / 7.5 | 7 / 7.5 | 8 / 7.5 | 8 / 7.5 | 7.5 / 7.5 |
| chat-no-selection | 6.5 / 7 | 7 / 7 | 8.5 / 8.5 | 8 / 8.5 | 8 / 9 | 8.5 / 8.5 | 8.5 / 9 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 |
| chat-filter-no-match | 5.5 / 6.5 | 7 / 7 | 8 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8 | 8 / 8.5 |
| group-chat | 6 / 7 | 7 / 7 | 8 / 8 | 8 / 8 | 7.5 / 8.5 | 8.5 / 8 | 8.5 / 8 | 8.5 / 8 | 8.5 / 8.5 | 8.5 / 8 | 8.5 / 8 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8 | 8 / 8.5 |
| channel | 6.5 / 7 | 7 / 7 | 7.5 / 8 | 7.5 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8.5 / 8 | 8 / 8.5 | 8 / 8.5 | 8 / 8 | 8 / 8 | 7.5 / 8 | 8 / 8.5 | 8 / 8 | 8 / 8 | 7.5 / 8 |
| channel-narrow | 5.5 / 5.5 | 6 / 6 | 7 / 7 | 6.5 / 7 | 7.5 / 7.5 | 7 / 7.5 | 7.5 / 7.5 | 7.5 / 7.5 | 7.5 / 8 | 7 / 8 | 7 / 7 | 7.5 / 7.5 | 7.5 / 8 | 7.5 / 7.5 | 7.5 / 7.5 | 8 / 7.5 | 7.5 / 7.5 |
| teams | 6.5 / 7 | 7.5 / 8 | 8 / 8.5 | 7.5 / 8.5 | 8 / 9 | 8.5 / 8.5 | 8 / 9 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 |
| calendar | 6.5 / 7 | 7.5 / 8 | 7 / 8 | 7 / 8 | 7.5 / 8 | 7 / 8 | 7.5 / 8 | 7 / 8 | 7.5 / 8.5 | 7 / 8 | 7.5 / 8 | 7 / 7.5 | 7.5 / 8 | 7 / 8 | 7.5 / 8 | 7.5 / 8 | 7.5 / 8 |
| calls | 6 / 6.5 | 6.5 / 7 | 6.5 / 7.5 | 7 / 8 | 7 / 8 | 7.5 / 8 | 7.5 / 8 | 7.5 / 8 | 7.5 / 8 | 7.5 / 8 | 8 / 8 | 8 / 7.5 | 8 / 8 | 7.5 / 8 | 8 / 8 | 8 / 8 | 8 / 8 |
| calls-narrow | 3.5 / 3.5 | 6 / 6 | 6 / 7 | 6.5 / 7 | 6.5 / 7.5 | 7 / 7.5 | 7 / 7.5 | 7 / 7.5 | 7 / 7.5 | 7.5 / 7.5 | 7 / 7 | 7.5 / 7 | 7.5 / 7.5 | 7 / 7.5 | 8 / 7.5 | 7.5 / 7.5 | 8 / 7.5 |
| composer-mention | 6.5 / 6.5 | 7 / 7 | 8 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 7.5 / 8.5 | 8.5 / 8.5 | 8 / 8 | 8 / 8 | 7.5 / 8 | 7.5 / 8.5 | 8.5 / 8 | 8 / 8 | 8 / 8.5 |
| composer-long-draft | 5.5 / 6 | 6 / 6.5 | 6.5 / 7.5 | 6.5 / 7.5 | 7 / 7.5 | 7 / 7 | 6.5 / 7.5 | 7.5 / 7.5 | 8 / 7.5 | 8 / 7.5 | 7.5 / 7 | 7.5 / 7.5 | 7 / 8 | 6.5 / 7.5 | 7.5 / 8 | 8 / 8 | 7.5 / 8 |
| composer-images | 3.5 / 5.5 | 6.5 / 7 | 6.5 / 7.5 | 6.5 / 7.5 | 7 / 8 | 7.5 / 7.5 | 7 / 8 | 7 / 7.5 | 7 / 8 | 7.5 / 7.5 | 8.5 / 7 | 8 / 8 | 8 / 8 | 7.5 / 8 | 7 / 7.5 | 7.5 / 7.5 | 8 / 7.5 |
| image-preview | 5.5 / 7 | 7 / 8 | 6.5 / 8 | 7 / 8 | 7 / 8.5 | 7.5 / 8 | 8 / 8.5 | 7.5 / 8 | 8 / 8 | 7.5 / 8.5 | 7.5 / 8 | 7 / 8 | 8 / 8.5 | 7.5 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8 / 8 |
| search-switcher | 6 / 6.5 | 7 / 7 | 7.5 / 8.5 | 7 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8 | 8.5 / 8.5 | 8 / 8.5 | 7.5 / 8.5 | 8 / 7 | 7.5 / 8.5 | 8.5 / 8.5 |
| new-chat | 6 / 6.5 | 7.5 / 8 | 7.5 / 8 | 7.5 / 7.5 | 8 / 8 | 8 / 7.5 | 8 / 8 | 8 / 8 | 8 / 7.5 | 8.5 / 8.5 | 8.5 / 7.5 | 8.5 / 8 | 8.5 / 8 | 8 / 8 | 8.5 / 8.5 | 8 / 8 | 8.5 / 8.5 |
| settings | 4.5 / 6 | 6 / 7.5 | 6 / 7.5 | 7.5 / 7 | 7.5 / 8 | 8 / 7.5 | 8 / 7.5 | 7.5 / 7.5 | 8 / 7.5 | 8 / 8 | 7.5 / 7.5 | 7.5 / 8 | 7.5 / 7.5 | 7.5 / 8 | 8 / 8 | 7.5 / 8 | 7.5 / 8 |
| shortcuts | 6 / 6.5 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 6.5 / 8 | 7.5 / 8.5 | 8.5 / 8.5 | 7.5 / 8 | 7.5 / 8.5 | 7 / 8 | 8 / 8.5 | 8.5 / 8 | 8 / 8.5 | 7.5 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 |
| status-message | 6.5 / 7 | 6.5 / 7 | 7.5 / 8 | 7 / 8 | 7 / 9 | 8 / 8.5 | 8 / 8 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 9 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 |
| account-menu | 5.5 / 6.5 | 6.5 / 7 | 8 / 8 | 7.5 / 8 | 8 / 8.5 | 8.5 / 8 | 8.5 / 8.5 | 8.5 / 8 | 8.5 / 8 | 8 / 8.5 | 8.5 / 8 | 8 / 8 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8 | 8 / 8.5 |
| permission-consent | 6.5 / 7 | 8 / 8 | 8 / 8 | 8 / 8 | 7.5 / 8.5 | 8.5 / 8 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 |
| error-toast | 5.5 / 6 | 5.5 / 6 | 5 / 6.5 | 7.5 / 8 | 7.5 / 8 | 7 / 8 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8 / 7 | 8.5 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 |
| first-sync-loading | 5 / 6.5 | 7 / 7 | 8.5 / 8.5 | 8 / 8.5 | 8 / 9 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 9 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 |
| first-sync-empty | 5.5 / 6 | 7 / 7 | 7.5 / 8.5 | 7.5 / 8.5 | 7 / 8.5 | 8 / 8.5 | 8 / 9 | 7.5 / 9 | 8 / 8.5 | 8 / 8.5 | 8 / 9 | 8 / 8.5 | 8.5 / 9 | 8 / 9 | 7.5 / 8.5 | 7.5 / 8.5 | 8 / 9 |
| call-bar | 6 / 6.5 | 5.5 / 7 | 6.5 / 7 | 6.5 / 7.5 | 7 / 8.5 | 7.5 / 8 | 7 / 8 | 8 / 8 | 7.5 / 8.5 | 7.5 / 8 | 7.5 / 7.5 | 8 / 8 | 7.5 / 8.5 | 7.5 / 8 | 8 / 8 | 8 / 8 | 7.5 / 8 |
| call-docked | 6 / 7 | 6 / 7 | 7 / 7.5 | 7.5 / 8 | 7.5 / 8.5 | 8 / 8 | 7.5 / 8 | 8 / 8.5 | 7.5 / 8.5 | 8.5 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8 / 7.5 |
| join-on-phone | 7 / 8 | 7.5 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8 / 9 | 8 / 8.5 | 8 / 9 | 8.5 / 8.5 | 8.5 / 8 | 8 / 9 | 7.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8 | 8 / 8.5 | 8.5 / 9 | 8.5 / 8.5 | 8 / 8.5 |
| setup | 6 / 6.5 | 6.5 / 7 | 7 / 7 | 7.5 / 8 | 7.5 / 8 | 7.5 / 7.5 | 7.5 / 7.5 | 8 / 7.5 | 7.5 / 7.5 | 7.5 / 7.5 | 8 / 7.5 | 7.5 / 7.5 | 7.5 / 8 | 7.5 / 8 | 7.5 / 8 | 8 / 7.5 | 8 / 8 |
| signin | 6.5 / 7 | 7.5 / 8 | 8 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8 / 8 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 |
| signin-browser | 6 / 7 | 7.5 / 8 | 8 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 |
| signin-device-code | 6 / 7 | 7.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 9 | 8.5 / 8 | 8.5 / 8.5 | 8.5 / 9 | 8.5 / 8.5 | 8.5 / 9 | 8.5 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 |
| signin-finishing | 6.5 / 6.5 | 8 / 8 | 8 / 8.5 | 8.5 / 8.5 | 8 / 9 | 8 / 8.5 | 8 / 8.5 | 7.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8 | 8.5 / 8 |
| signin-expired | 6.5 / 7 | 7.5 / 8 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8 / 8 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 |
| signin-failed | 6.5 / 7 | 7.5 / 8 | 8 / 8.5 | 8 / 8.5 | 8 / 9 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 |

## Dimension scores, rounds 0-8

| Dimension | R0 | R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| hierarchy | 7 / 7 | 7.5 / 7 | 7.5 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 8.5 / 8 | 8.5 / 8 |
| spacing | 6 / 6 | 6.5 / 6.5 | 7 / 7.5 | 7 / 7.5 | 7 / 8 | 7.5 / 7.5 | 7.5 / 8 | 7.5 / 8 | 7.5 / 8 |
| typography | 6.5 / 6.5 | 6.5 / 7 | 6.5 / 7.5 | 7.5 / 7.5 | 7 / 8 | 7 / 8 | 8 / 7.5 | 8.5 / 8 | 8 / 7.5 |
| consistency | 5.5 / 7.5 | 7 / 8 | 6.5 / 8 | 7 / 8.5 | 6.5 / 8.5 | 7.5 / 8.5 | 7.5 / 8.5 | 7.5 / 8.5 | 7.5 / 9 |
| contrast_accessibility | 4.5 / 4 | 6.5 / 6 | 7.5 / 8 | 8 / 8 | 8 / 8 | 8 / 8.5 | 8.5 / 8 | 8.5 / 8.5 | 8.5 / 8.5 |
| feedback | 6 / 6 | 6.5 / 6.5 | 7 / 7 | 7.5 / 8 | 8 / 8 | 8 / 7.5 | 7.5 / 8 | 8.5 / 7.5 | 8.5 / 8 |
| states | 5.5 / 6.5 | 7.5 / 8 | 7.5 / 8 | 8 / 8.5 | 8 / 8.5 | 8.5 / 8.5 | 8.5 / 8.5 | 8 / 8.5 | 8.5 / 8.5 |
| navigation | 6.5 / 6.5 | 7 / 7 | 7.5 / 7.5 | 7 / 8 | 7.5 / 8 | 8 / 8 | 8 / 8 | 8 / 8 | 7.5 / 8 |

## Rounds

### Round 0

Scores: Opus 6.0, Codex 6.5.

- Baseline after removing every theme but light, dark and System.

Top findings:

- Opus, high: The tertiary text colour fails WCAG 4.5:1 everywhere: timestamps, list dates, section labels, helper text, emails, hints and footers.
- Opus, high: The light accent #c26318 is too light for text and for white button labels. Light-mode success green also fails.
- Opus, high: At 760pt the Calls page clips horizontally: the intro text, the Join button and the capability descriptions are cut off.
- Opus, high: Pasted-image thumbnails render broken: mixed sizes, bad crops, a clipped strip and detached remove buttons.
- Opus, high: Settings dialog is broken: 'Leave demo' is clipped by the scrollbar, unchecked checkboxes are nearly invisible, and dropdown labels sit after their controls.
- Codex, high: Narrow Calls layout clips its primary task.
- Codex, high: Muted informational text repeatedly fails the 4.5:1 text requirement.
- Codex, high: White text on orange primary buttons fails body-text contrast.
- Codex, high: Fixed sidebar and hover action placement compromise message reading.
- Codex, medium: Orange small text fails contrast on white and tinted surfaces.

### Round 1

Scores: Opus 7.0, Codex 7.0.

- Colour tokens: secondary and dim text reach 4.5:1 on every surface in both modes, including selected rows. The light accent darkens from #c26318 to #a9530f, so white button labels pass. Success, warning and danger pass too. A unit test enforces all of it.
- A new `control` token gives text fields, checkboxes and secondary buttons a 3:1 edge, and focused fields get a 2 pt accent ring.
- Avatar colours are re-tuned so white initials reach 4.5:1.
- Shortcut hints use the platform's notation everywhere (Ctrl+K on Linux and Windows, ⌘K on macOS). The shortcuts sheet shows key caps and a close button.
- Empty states fit inside narrow panes (the sidebar no longer widens) and offer the next step: New chat on a first start, Clear filter when nothing matches, skeleton rows while chats load, and Teams-specific copy and icons.
- Settings: content clears the scroll bar, drop-down labels sit above their controls, and the zoom hint is quieter. All dialog titles share one size (20 pt bold).
- Calls: the page reflows at narrow widths instead of clipping, capabilities are listed with icons, Join is the primary button, and the copy is fixed ("in its own Edge window").
- Composer: pasted-image thumbnails sit on tiles with a backdrop and border, with the remove button on each tile's corner. Message rows tint on hover. Mention rows get a 3 pt accent edge, and the dark mention tint is calmer.
- Sign-in: a spinner while waiting for the browser or code, the device code in a frame with a clickable link, Try again after a failure, and the account shown on its own line when a session expires. The demo button is hidden while sign-in runs. The setup form explains why Save is disabled.

Top findings:

- Opus, high: Placeholder text fails WCAG 1.4.3 in light mode in every input (composer, filter, search, status, setup)
- Opus, high: Failed send is shown only as a transient toast that covers the last message, sits off-centre, and has no Retry
- Opus, high: Settings checkboxes look like radio buttons and the unchecked state fails 3:1
- Opus, high: Call bar contradicts the chat and wastes space
- Opus, medium: Hover action toolbar covers message author lines and reactions
- Codex, high: Narrow layouts preserve a desktop-width sidebar at the expense of the task pane.
- Codex, high: Tinted-input placeholder text narrowly fails WCAG 2.2 body-text contrast.
- Codex, high: White-field placeholder text also fails the 4.5:1 threshold.
- Codex, high: Failed-send feedback does not provide a visible recovery path tied to the failed message.
- Codex, high: Hover action strips obscure sender metadata.

### Round 2

Scores: Opus 7.0, Codex 7.5.

- Placeholders use the dim token, so they reach 4.5:1. The composer gets a 3:1 edge and a 2 pt accent focus ring. Text fields and the Join field match the 32 pt button height.
- A failed send is marked on the message itself: an alert icon, Not sent and a Retry button. The demo's error state now reproduces a real failed send, not a toast alone.
- Settings uses square checkboxes with a 3:1 edge, an accent fill when checked and a focus ring, replacing egui's round ones. A UI test covers toggling from the label.
- Primary buttons are 32 pt, the same as secondary ones. Disabled, they turn neutral instead of faded orange.
- Narrow windows: below 960 pt the sidebar narrows to 232 pt without overwriting the saved width, and the hover toolbar shows three reactions in narrow panes. Wide windows: the message column stops at 920 pt.
- Presence dots differ by shape as well as colour: a bar for Do not disturb, a hollow centre for Away and Be right back, a ring for Offline.
- Call bar: title and status sit together, and Done becomes a neutral Close bar.
- A hairline under the conversation header. Dark dialogs get a lighter edge. Account-menu rows share one icon column, with a badge for other accounts' unread chats. The switcher footer shows key caps. Empty states are centred. Calls puts Join first.
- The long-draft composer keeps Send and Attach at the bottom and shows its scroll bar.

Top findings:

- Opus, high: The error toast covers the failed message, including its 'Not sent' label and Retry button, which are the recovery path it describes.
- Opus, high: 'Show me as available while I use Wren' renders a ☑ text glyph instead of the checkbox widget, misaligned with every other checkbox.
- Opus, medium: Hyphens render with wide side bearings across the app, so 'On-call' reads 'On - call', and 'sign-in', 'go/no-go', 'full-text', 'third-party' and 'F4KQ-9TZW' are affected too.
- Opus, medium: Selected rows are shown only by a near-invisible fill, which fails the 3:1 non-text contrast guidance for a state indicator.
- Opus, medium: The Calendar and Calls detail panes have no top header bar, while Chat and Channel have a 52 pt bar with title and actions. The frame jumps between sections.
- Codex, high: Error toast obscures the failed message and its recovery action.
- Codex, medium: Fixed navigation consumes too much of a narrow window and truncates identities.
- Codex, medium: Live call status conflicts with the visible ended-meeting state.
- Codex, medium: The first-run hierarchy foregrounds technical configuration over the step that makes it possible.
- Codex, medium: Calls screen reads like a capability document instead of an action surface.

### Round 3

Scores: Opus 7.5, Codex 7.8.

- Toasts appear at the top of the window, so a failed message and its Retry button stay visible above the composer.
- Calendar and Calls share the 52 pt header bar that conversations use: title, subtitle, right-aligned actions and a hairline. The meeting's Join and Copy link move into it.
- Selected chats, channels and switcher results get a 3 pt accent bar, so selection no longer rests on a fill alone. Switcher results show initials for people and chats.
- The composer matches the message column's width. Its hint is 12 pt and shorter.
- Calls: two lines on how calls work, a one-line summary of what works and what doesn't, and the details behind a disclosure.
- Channel posts get a Reply button with an icon. The last Settings checkbox now uses the shared widget. The status dialog enables Save only after a change and offers Remove status only when one is set. Setup shows the registration guide before the fields. Settings copy is plainer.
- Demo: joining the meeting adds today's Meeting started line, so the call bar no longer contradicts yesterday's Meeting ended.

Top findings:

- Opus, high: Narrow layouts do not adapt: the list pane stays ~232 px, names truncate, the composer placeholder wraps with a dangling '·' and the helper wraps onto 2-3 lines.
- Opus, high: Call layouts truncate the chat title early and remove all navigation.
- Opus, high: Long drafts are cramped and clipped: tight leading, no top padding, scrollbar beside the attach icon.
- Opus, medium: Secondary buttons, reaction chips, hover toolbar and cards rely on borders far below 3:1, so secondary controls barely read as buttons, especially in dark mode.
- Opus, medium: Avatar colours and icons in the switcher do not match the chat list, breaking recognition.
- Codex, medium: Fixed navigation widths squeeze the task pane and truncate identity information.
- Codex, medium: The scrollbar thumb and track do not meet the 3:1 non-text contrast threshold.
- Codex, medium: Helper and metadata text is too small for comfortable desktop reading, even though sampled colors pass contrast.
- Codex, medium: Reading and composing use different horizontal constraints on wide windows.
- Codex, medium: The long-draft viewport cuts through the first visible line and packs many lines tightly against its edges.

### Round 4

Scores: Opus 7.5, Codex 8.0.

- The composer now stops at the message column on wide windows (round 3's cap had no effect: the bottom panel stretches its Ui, so the composer now draws into a sized child). It uses a 20 pt line height, and its hints shorten in narrow panes so they stay on one line.
- The switcher draws each chat with the chat list's own avatar (person, group initials or meeting tile), so colours and icons match.
- Secondary buttons get the 3:1 control edge. Drop-downs and disclosures use Lucide chevrons. The Settings scroll bar is 8 pt with a 3:1 thumb.
- The status and permission dialogs get close buttons. The permission dialog leads with the capability and puts the Graph scope last, in small print.
- Failed messages get a light danger tint. The conversation title uses the width its header buttons leave instead of a fixed 260 pt reserve, so docked meeting titles no longer truncate early.
- The shortcuts sheet is grouped (Navigation, Messages, App), and the account menu gets Accounts and Status headings.
- First start: the list shows no filter field or duplicate empty text while there's nothing to list. The sign-in page sits higher, so setup fits without scrolling.

Top findings:

- Opus, high: Buttons have no single system: filled primaries render ~15pt bold while outlined secondaries are 13pt; secondary buttons come in three styles (outlined #8a8d95, filled borderless pill #eeeef0 e.g. 'Copy link'/'Restore', and faint-bordered #dfdfe3 'Reply'/reaction chips); disabled Save/Join are 15pt grey slabs.
- Opus, high: Message hover toolbar overlaps neighbouring rows (mention highlight above, previous reaction chips, the post's own name row).
- Opus, medium: Shortcut descriptions are not aligned on one column; each section sizes its own key column.
- Opus, medium: Hyphens render with wide side bearings across the app ('On - call', 'go/no - go', 'Step - by - step', 'Sign - in', 'F4KQ - 9TZW').
- Opus, medium: Interactive outlines below 3:1: hover toolbar, reaction chips and Reply buttons.
- Codex, medium: Reply-button borders do not meet the requested 3:1 component-boundary contrast. Their labels remain readable.
- Codex, medium: Narrow layouts spend too much width on navigation while still truncating important identities.
- Codex, medium: The scrolled draft clips the first visible line against the editor's top edge.
- Codex, medium: Pasted-image thumbnails crop away content needed to verify attachments before sending.
- Codex, medium: People search loses its purpose label once the user types; the existing-chat explanation is too compressed.

### Round 5

Scores: Opus 7.5, Codex 8.0.

- One button style: primary and secondary buttons share 13 pt semibold labels and a 32 pt height. Reply buttons, reaction chips and the hover toolbar get the 3:1 control edge.
- The hover toolbar sits inside the hovered row instead of across its neighbours. On wide windows the conversation header's actions end where the message column ends.
- Toasts centre on the main pane, not the window. The sidebar keeps its width when its list is empty. The header spinner stays hidden while the first chat list loads, since the pane shows progress.
- A scrolled long draft shows whole lines: the editor's height is a whole number of 20 pt lines. The composer hint is shorter.
- The shortcuts sheet is one grid, so descriptions align across groups. New chat labels its field (To / Add someone else). Email addresses in mention suggestions and people search are 12 pt.
- Calls: the link field matches the Join button's height. Summary lines hang under their text. Limits read as neutral information, not as errors.
- Image preview uses a segmented Fit / 100% control. Pasted images show their dimensions. The status dialog uses a one-line field with Cancel and Save on the right, and Enter saves. Be right back shows clock hands. The sign-in group is centred vertically.

Top findings:

- Opus, medium: Hyphens render with wide side gaps across the app ('On - call', 'Sign - in', 'Ad - hoc', 'go/no - go', 'Step - by - step', 'F4KQ - 9TZW', 'open - source').
- Opus, medium: At 1920 wide the conversation column is capped at about 920 px and left-aligned, leaving a ~600 px dead band. Header actions stop at x≈1290 while the header rule runs to 1920.
- Opus, medium: Error toast is placed over the conversation's top message and covers its text. It also repeats the inline failure without its own action.
- Opus, medium: The orange left bar marks the selected row in Chat/Teams lists but appears on every Calendar row. Calendar selection is only a fill with 1.17:1 against the list.
- Opus, medium: Mixed button language: filled pills with almost no boundary (Copy link, call bar buttons, theme segment) sit next to 6 px-radius outlined buttons (Reply, Cancel, Leave demo). Pill buttons fail 3:1 for component boundaries.
- Codex, high: Long draft hides the send and attachment controls.
- Codex, medium: Narrow layout preserves too much navigation width and truncates important identities.
- Codex, medium: Attachment crops make extreme-aspect images hard to recognize.
- Codex, medium: Wide conversation is left-heavy and its header does not align with its content column.
- Codex, medium: Technical setup lacks a strong guided action hierarchy.

### Round 6

Scores: Opus 8.0, Codex 8.0.

- On wide windows the conversation header, history and composer share one centred 920 pt reading column.
- A failed send keeps its reason on the message (Not sent: reason, then Retry). The toast appears only when the failure is in a chat that isn't open, and retrying clears the reason. The new Message.failure field is optional, so existing caches keep loading. A unit test covers both paths.
- Calendar rows mark the kind of meeting with a dot (live, online, other), leaving the accent bar to mean selection everywhere.
- Pill buttons (Copy link, the call bar, the theme choice) get the 3:1 control edge.
- History fades out under the header instead of cutting messages off, and the hover toolbar no longer shows under dialogs.
- Setup: the registration guide is a button, and Save reads Save and continue on first run. The code sign-in tooltip is one plain sentence. The status field is labelled.

Top findings:

- Opus, high: Long draft composer shows clipped glyph fragments at the top edge and a stray scrollbar stub beside the attach icon; no caret visible.
- Opus, high: Call bar duplicates the meeting title, has no leave/hang-up or duration, and the 'Restore' and 'Close bar' labels are ambiguous; the 'Meeting ended' line attaches to the preceding message.
- Opus, medium: Text fields don't show focus. Only the composer gets the 2pt accent ring, so a field that is being typed in looks idle.
- Opus, medium: Pending-image thumbnails are centre-cropped, so wide or tall images are misrepresented, and remove buttons cover the image corners.
- Opus, medium: At 760pt width the list pane keeps its full 232pt; names truncate heavily and the conversation gets only 464pt.
- Codex, medium: Narrow layouts sacrifice identity and reading space before adapting the secondary pane.
- Codex, medium: Hover toolbars present too many equally weighted, tightly packed commands.
- Codex, medium: Small supporting text carries consequential instructions and state.
- Codex, medium: Explanatory prose crowds the task and weakens progressive disclosure.
- Codex, medium: Active-call controls do not clearly explain the consequence of closing the bar.

### Round 7

Scores: Opus 8.0, Codex 8.0.

- Capture fix: screenshot mode treats the window as focused, so focus rings and the caret show as they do in use. Under Xvfb with no window manager the window never gets focus, and egui only draws focus while the window has it. The composer's own focus ring follows window focus the same way egui's fields do.
- A scrolled long draft fades out at its top edge instead of showing a cut line. Its scroll bar no longer sits as a stub next to the icons.
- Docked beside a call, the call bar shows only the call's state, since the meeting chat's header right below names the meeting. Close bar reads Hide call bar (docs/calling.md updated).
- System lines (Meeting started, Meeting ended) get room above and below.
- The hover toolbar separates reactions from message actions with a divider.
- A narrow Calls list keeps names whole by dropping the video button. Video calls stay one click away in the chat header.
- The Settings note on when messages refresh is 13 pt in the secondary colour.

Top findings:

- Opus, medium: Primary (accent) buttons carry the neutral control stroke around the accent fill, so their edge reads muddy; the calendar header Join pill has no stroke, so primaries are inconsistent.
- Opus, medium: Narrow (760pt) layouts keep a 232pt side pane, truncating names and wrapping posts at about 380pt. Calls silently drops the video button and the hover toolbar collides with header rows.
- Opus, medium: At 1920x1080 the conversation is top-anchored, leaving a large dead gap above the composer; chat apps bottom-anchor short histories.
- Opus, medium: The settings scroll area cuts a row mid-glyph at the bottom edge, and the background-refresh help paragraph is attached to the tray option instead of notifications.
- Opus, medium: Meeting detail is thin and oddly placed: a fixed 640pt left-aligned card, redundant Location/Online rows, no attendees, and header buttons in a different shape and size from the app's buttons.
- Codex, medium: Narrow layouts reserve too much width for the sidebar and truncate identities.
- Codex, medium: Video calling loses its visible entry point at narrow width.
- Codex, medium: Attachment thumbnails conceal the contents of extreme aspect-ratio images.
- Codex, medium: Long draft overflow has no visible position cue.
- Codex, medium: Search highlight and committed recipients are easy to confuse.

### Round 8

Scores: Opus 8.0, Codex 8.0.

- Primary buttons drop egui's grey control edge. Pills are 32 pt, like every other button.
- A conversation shorter than its pane sits at the bottom, next to the composer, instead of leaving a gap above it.
- New chat marks the keyboard's choice with the accent bar and an Enter to add hint, so it reads differently from people already added.
- A narrow Calls list keeps both call buttons, made smaller.
- Calendar leaves out a Location that only repeats the Online row.
- Settings puts the note on when messages arrive under the notifications option it explains.

Top findings:

- Opus, high: Side list keeps its full 232pt width in a 760pt window, so names truncate after 9-12 characters and the conversation column shrinks to ~460pt.
- Opus, medium: Detail pages use left-aligned fixed-width cards with different widths (Calendar 640pt, Calls 680pt) while conversations use a centred column, so pages feel unrelated and leave ~250pt dead space on the right.
- Opus, medium: Dialog footers are inconsistent: primary bottom-left with helper text (New chat), primary-left then secondary (Permission), right-aligned Cancel/Save (Status message), lone left-aligned secondary (Join on phone).
- Opus, medium: In meeting mode the navigation rail and chat list vanish, so there is no way to answer another chat without leaving the meeting layout; 'Restore' is ambiguous.
- Opus, medium: Pasted-image thumbnails are cropped to fill, so wide or tall images are unrecognisable, and the remove buttons sit on the image.
- Codex, medium: Narrow layouts truncate people’s identities while retaining permanent navigation and action columns.
- Codex, medium: The long draft has no clear indication of its hidden beginning.
- Codex, medium: The active search result and primary action refer to different people.
- Codex, medium: Settings is a long flat scroll surface with weak wayfinding.
- Codex, medium: Technical setup instructions dilute the main task.

### Round 9

Scores: Opus 8.0, Codex 8.0.

- Contrast headroom: control edges move to #7c7f87 light and #82858e dark, and now keep 3:1 on every surface, selected rows included. Danger red (#b42525 / #f58a8a) keeps 4.5:1 on every surface. The contrast test enforces both.
- Running text (sign-in, setup, empty states, settings and dialog notes) uses a 1.45 line height (theme::leading).
- Calendar and Calls sit in a 720 pt column, and the conversation column narrows to 860 pt.
- Dialog footers share one pattern: note above, actions on the right, primary last (New chat, permission, Join on phone).
- A failed send also shows Not sent in the chat list. The shortcuts sheet has two columns in one grid. Settings notes are indented under their checkbox. The mention list has a People heading and a key hint. Join on phone names the meeting. Setup keeps environment variables under Advanced. The call bar's Restore reads Full window.
- Tried and kept out: drawing captures with the system font (Noto Sans) instead of the bundled Inter. The hyphen improved but the weights got weaker.

What each grader said blocks a 9:

- Opus: Dialog chrome is not a shared component: close-button and padding positions vary, with the shortcuts close button ~100 px off its edge
- Opus: Scrolled content is sliced under headers (half avatars, cut card borders, 2.1:1 faded text lines)
- Opus: Calendar and Calls detail panes are sparse and off the page grid (headers at x 462, input/button heights 28 vs 32 px)
- Opus: Narrow (760 px) layouts keep full sidebars and truncate names heavily
- Opus: Inconsistent running-text leading (17/20/23 px) and control heights across sign-in, setup and forms
- Codex: Narrow layouts truncate identities while keeping a costly sidebar and fixed action gutters.
- Codex: Attachment crops conceal image content and long drafts lack visible scroll/length feedback.
- Codex: Reading hierarchy needs refinement: long message lines, small metadata, dense settings/setup copy and uneven use of empty space.

### Round 10

Scores: Opus 8.0, Codex 7.5.

- One dialog frame: every dialog has 24 pt padding, and the shared modal draws the close button at the same corner, level with the title. The shortcuts sheet's close button had been 100 px off its edge.
- Calendar and Calls start where conversation headers start (30 pt in). The meeting header says how soon it starts, and the invitation text has its own label.
- Calmer accent: orange is kept for actions, selection and mentions. Own names, the DEMO badge and calendar dots are neutral, and the light accent darkens to #9c4b0c (6.1:1 on white).
- Pasted images fit their tile whole, with a 12 pt minimum short side. The UI tests now check proportions and the minimum instead of the crop.
- Text fields are 32 pt like buttons, message prose stops at 680 pt, and message hover is a visible tint. The fade under headers is taller and eased, and a long draft always shows its scroll thumb.
- Shortcut notation is consistent (Shift+Enter, Alt+Up). The notifications note is one line, with the detail in a tooltip, and tooltips wrap at 280 pt. Narrow call bars use icon buttons, and narrow Calls rows show their call buttons on hover. Image preview sizes its canvas to the image. Setup fits without scrolling.

What each grader said blocks a 9:

- Opus: Narrow (760 px) layouts are not adapted: the 232 px list column stays fixed, names truncate to 'Katherine Jo...', the hover toolbar is clipped under the header in channel-narrow, and the per-person call/video buttons disappear in calls-narrow
- Opus: Light mode: the quote block background (#eeeef0) is identical to the hovered-message background (#eeeef0), so the quote card vanishes on hover (chats, chat-filter-no-match); dark keeps them distinct (#2c2d33 vs #25262b)
- Opus: Inconsistent rhythm in the same component: card body leading is 20 px (signin, setup), 23 px (signin-device-code) and 17 px (signin-finishing); button side padding is 16 px ('Sign in with Microsoft') vs 9-10 px ('Open the page again', 'Copy link')
- Opus: Large dead bands: ~100 px empty above the conversation in chats-wide and ~200 px in call-bar/call-docked, while the image preview and dialogs hug the top (y=25 to 120) instead of sitting at a consistent optical centre
- Opus: Stray-looking chrome: a 3x14 px detached scrollbar thumb beside the attach icon in composer-long-draft reads as a caret; the settings dialog shows a heavy 6 px #7c7f87 thumb and a half-clipped checkbox at its bottom edge with no fade
- Codex: Scroll-edge fades wash out still-visible message text; measured contrast fails in five screens.
- Codex: Narrow layouts retain a large sidebar, truncate identities and remove visible call actions.
- Codex: Message hover toolbars give many reactions and secondary actions excessive visual weight.
- Codex: Settings needs clearer category navigation and clean clipping; some selection and setup instructions need more precise hierarchy and wording.
- Codex: Small metadata, variable text measure and image identification leave clear readability and polish work.

### Round 11

Scores: Opus 8.0, Codex 8.0.

- Removed the scroll-edge fades. Codex measured the text they half-covered failing contrast. History and drafts now clip cleanly at their edges.
- The message toolbar shows three reactions, Reply and More, with a quiet outline. More holds the other reactions plus Copy, Edit and Delete, and the toolbar stays up while its menu is open.
- Light-mode quotes get a darker fill than the hover tint, so they no longer vanish on hover.
- Rhythm: one button padding (14 pt sides, no padding spaces in labels). Sign-in cards share one title and body style. Menus and tooltips get 10 × 6 pt insets, and setup field groups stand apart.
- Dialog placement: static dialogs sit a little above the middle, by their own height. Search, New chat and image preview keep a fixed top, so their fields never move while you type; a UI test caught the switcher jumping.
- Colour: the light accent is a cleaner #a5500f, and Away / Be right back amber becomes gold #8f6a00, clear of it in hue.
- Details: a short history with nothing older says so at the top. Calendar lists meetings in time order. A failed send previews its own text in the chat list. The selected row's time uses the secondary colour. Small avatars keep 11 pt initials, and team tiles use 12 pt.
- Narrow call bars use short labels, and Hide uses an eye-off icon. Narrow Calls rows keep both call buttons visible again. Image preview grows to the window less 48 pt margins. Long drafts show a full-height scroll track.

What each grader said blocks a 9:

- Opus: Overlays use inconsistent geometry: the image preview is a short 260 px strip pinned to the top (not a full-window viewer), and dialog tops jump between y=48, 90, 193 and 207 depending on the dialog
- Opus: Narrow layouts (760 wide) keep the list pane at 300 px, so names truncate after about 10 characters ('Katherine Jo...', 'Grace Hop...') while the main pane keeps generous margins
- Opus: Calendar detail is underdesigned: a two-row card (Organizer/Online), a 15 pt title that matches the list items, an irrelevant join note for an offline event, and about 75% of the pane empty
- Opus: Scroll affordances are rough: always-visible 6 px grey bars (#727376) on the right edge (setup page overflows at 1280x800 and pushes the logo to y=20), a heavy black thumb plus a hairline track inside the long-draft composer, and settings content cut through a checkbox with no fade
- Opus: Surface layering in dark mode is inverted: the list pane (#1d1e22) is lighter than the main pane (#17181b), and in light mode the list and main panes are separated only by a 1.08:1 tint with no divider
- Codex: Narrow navigation consumes too much space and truncates identities; wide layouts leave the reading column detached from the list.
- Codex: Supporting text is consistently small, while calendar and Calls lack a strong task-first detail hierarchy.
- Codex: Polish is uneven in recipient selection, first-run composition, composer clipping/thumbnails, and dialog spacing.

### Round 12

Scores: Opus 8.0, Codex 8.0.

- Image preview is a viewer: the window less a 32 pt inset, on a darker media scrim. Its Fit / 100% toggle is neutral, not orange.
- Dialog placement: the search switcher sits at 12% of the window height, and New chat at a fixed, centred top, so neither moves as results change. Other dialogs centre by their height.
- Layering: the line between sidebar and main pane had been invisible, because the main pane painted over it; it is now drawn half a point inside the sidebar. In dark mode the sidebar is recessed (#131417) and the rail darker still.
- Less orange: your own reactions are marked by a warm tint and a firmer neutral edge. Channel cards get a firmer card edge, and Reply becomes a quiet text action.
- Calendar and Calls titles are 18 pt. Offline events drop the Online row and the joining note, and the calendar demo page now selects an online meeting.
- Settings uses a slim floating scroll bar in a reserved gutter, and setup uses a smaller logo. While the first chat list loads it shows its filter (disabled), so nothing moves when chats arrive. The device-login link is underlined.

What each grader said blocks a 9:

- Opus: Scroll areas hard-clip content under headers and at dialog bottoms (half lines, half avatars, chip slivers) with no fade
- Opus: Layout inconsistencies in content containers: ragged channel card edges, mention popup touching the composer, settings scrollbar crowding controls
- Opus: Thin secondary screens: calendar detail is sparse and left-weighted with three different left edges; meeting chat has large empty bands
- Opus: Mismatched component styling: composer scrollbar, focus ring on the filter field, two link styles, tooltip overflowing its card
- Opus: Surface ladders too flat (light hover/selected ~1.16:1, dark rail/list/main 1.04:1), so states and regions blur
- Codex: Narrow sidebars truncate person names and squeeze conversation content at 760 px.
- Codex: Important supporting text is repeatedly only 11-12 points, and onboarding footers use extremely long lines.
- Codex: Spacing and hierarchy need another pass: settings ends through a control, meeting times are understated, and short call transcripts float below large blank areas.
- Codex: Thread grouping and preview selection are too subtle; attachment previews and the dark QR quiet zone need refinement.

### Round 13

Scores: Opus 8.0, Codex 8.0.

- A surface ladder with visible steps. Light: hover #f0f0f2, selected #e2e3e8. Dark: rail, list, main pane and raised surfaces each a step apart. Channel cards get firmer edges. The contrast test still passes.
- Scroll shadows: where content continues past an edge (history under the header, Settings top and bottom), a soft shadow shows it. A shadow darkens the background slightly instead of fading text, so the text keeps its contrast.
- One link colour: override_text_color had been painting ui.link in body text colour, so links now set the link colour explicitly.
- Larger supporting text: message times and the composer hint are 13 pt, field labels 13 pt medium, and the onboarding footer 13 pt wrapped at 560 pt. Message prose stops at 640 pt, and page-header meeting times are 13 pt medium.
- Alignment and details: Calendar and Calls header actions sit at the pane's right edge. The mention list stands 8 pt clear of the composer. Channel cards share one whole-point width. Setup uses a smaller logo and starts where the other sign-in screens do. The QR code has a four-module quiet zone. New chat recipients are larger chips. The image viewer marks the chosen zoom, and the composer's scroll thumb is slimmer.

What each grader said blocks a 9:

- Opus: Narrow windows (760 px) keep the full 230 px list column, so chat and call names truncate to 'Katherine Jo…', 'Dorothy Vaug…', 'Grace Hop…' and the conversation pane gets only ~465 px
- Opus: Composer with a long draft shows a stray grey scrollbar (x=1166-1170, y=730-750) beside the attach icon, and the top line of text is clipped (descender fragments at y=553) with no inner padding or fade
- Opus: Dark mode hover toolbar has the same fill as the hovered row (#26272c on #26272c, 1.00:1), so the toolbar has no surface of its own and is held only by a 1.33:1 border
- Opus: Surface ladder is almost flat in light mode: rail #ececee / sidebar #f6f6f7 / main #fff differ by 1.08-1.09:1, the selected chat row is 1.19:1 against the sidebar, and card borders are #dfdfe3 (1.33:1), so structure relies on hairlines and the accent bar
- Opus: Accent orange does too many jobs (unread time, unread dot, badge, selection bar, focus ring, primary buttons, checkboxes, mention, segmented control). In light mode #a5500f reads as brown rather than as the bright #e88c3e of dark mode
- Codex: Narrow layouts truncate identities and spend almost 39% of a 760pt window on navigation.
- Codex: The long-draft composer shows a clipped preceding text line and a weak scroll indicator.
- Codex: Useful secondary information is repeatedly set at approximately 11-12pt, with insufficient typographic separation in dense dialogs.
- Codex: Meeting detail, calling explanation and recipient selection need clearer action hierarchy.
- Codex: Scroll cues, attachment identity and hover treatment still need refinement.

### Round 14

Scores: Opus 8.0, Codex 8.0.

- Colour: the light accent is a clear orange, #b4530a (5.0:1 with white labels), rather than a brown. Unread times are strong neutral text, leaving the dot to carry the accent.
- The message toolbar gets its own raised surface in dark mode, plus a card-edge border and a soft shadow in both modes.
- The light sidebar and rail step down a little more (#f3f4f6, #e7e8eb), and card edges are firmer.
- A scrolled draft covers the descender fragments of the line above with the field's own colour, and its scroll thumb is at least 40 pt.
- Failed messages carry a red edge bar like mentions do, and Retry is 28 pt.
- Popover rows have more room: mention rows are 44 pt with 28 pt avatars, and switcher rows 52 pt with 13 pt previews. Account emails are 13 pt.
- The shortcuts sheet uses one grid per column, so its headings keep one rhythm. New chat labels its recipients. A narrow Calls list keeps the audio call visible and shows video on hover. Setup field groups get 16 pt between them.

What each grader said blocks a 9:

- Opus: Component boundaries below 3:1: channel post cards and the floating message toolbar use #b9bcc4 on #ffffff (1.90:1) in light and #4d4f58 on #1a1b1f (2.11:1) in dark; keycaps are 1.33:1 light, 1.15:1 dark
- Opus: The selected nav-rail tile is invisible in light mode (#e2e3e8 on #e7e8eb, 1.05:1); dark mode shows it (1.59:1), so the two themes differ
- Opus: Stray or heavy scrollbars: setup shows a permanent full-height 6px bar at the right edge (#717375 light, #959698 dark); the chat history bar is always solid; a long composer draft shows a loose scrollbar stub beside the attach icon
- Opus: Composer image thumbnails for very wide or tall images show as unreadable strips or a 1px column, and the tile (#f0f0f2) does not stand out from the composer (#ffffff, 1.13:1)
- Opus: Uneven vertical rhythm across dialogs and pages: in shortcuts, a section heading sits 28px above its first row in NAVIGATION but 14px in APP; settings checkbox rows are spaced unevenly; the setup card title is 8px from its body; empty states sit at y=316 on one screen and y=340 on others
- Codex: The bottom scroll fade is painted over a real result preview, making text fail WCAG 2.2 AA.
- Codex: The narrow layout preserves too much navigation width and loses useful conversation or contact information.
- Codex: The narrow calls list loses the visible video-call action.
- Codex: Too much useful information uses the smallest text tiers.
- Codex: Extreme-aspect attachment previews are too small to inspect.

### Round 15

Scores: Opus 8.0, Codex 8.0.

- egui's default scroll-edge fade is off (fade strength 0). It was the source of the faded text Codex kept measuring below 4.5:1, most recently on the last visible search result. Edges now clip cleanly, and Wren's own shadows mark content beyond.
- Boundaries: the message toolbar's edge is the 3:1 control colour, and keycaps use the card edge. In light mode the selected rail tile is raised to white with an edge; it had been 1.05:1 against the rail.
- Calendar header actions sit over the content column with 8 pt between them. The search switcher draws a divider above its key hints.
- Sign-in screens centre by their own measured height, so the setup form no longer overflows into a permanent scroll bar.
- A narrow Calls list keeps both call buttons visible again.

What each grader said blocks a 9:

- Opus: Spacing and alignment are inconsistent between screens: the search switcher close X sits 9 px below the search row, the shortcuts section headers have 41 px vs 28 px gaps, empty states sit at different heights (icon centre at y316 vs y340), a 'Meeting ended' system line sits 15 px under a message where other system lines get about 30 px, and group chat leaves an empty band of about 70 px above the first date divider
- Opus: One orange accent does too many jobs: unread dots, nav badges, the selected-row bar, focus rings, mentions, quote bars, checkboxes, the send icon, primary buttons and meeting icons. That weakens it as a signal. In light mode the accent (#b4530a) reads as rust-brown, but dark mode's #e88c3e reads as tangerine, so the two themes have different personalities
- Opus: On wide windows, Calendar and Calls cap content at about 720 px, and the header actions (Copy link/Join end at x1114) float mid-window. The detail panes look empty and unfinished
- Opus: Settings dialog: the bottom scroll shadow renders as a hard grey band (#e5e5e5 line at y695) over half-hidden content, plus about 25 px of dead padding below it; it is a long single column with mixed control styles
- Opus: Narrow and edge layouts are unpolished: in calls-narrow, names truncate to 'Dorothy Vaug…' against the call icons. Extreme-aspect image thumbnails in the composer are unreadable strips. The first-sync-empty sidebar is completely blank
- Codex: Narrow layouts retain oversized sidebars and nested padding, causing avoidable name truncation and dense message wrapping.
- Codex: Secondary text is too small across metadata, settings, suggestions and error recovery.
- Codex: Conversation spacing and shifting sign-in layouts lack the stable rhythm expected of an exemplary native app.
- Codex: Attachment thumbnails and the pending-recipient state need clearer visual treatment.
- Codex: Thread boundaries are weak and light mention text only narrowly clears AA.

### Round 16

Scores: Opus 8.0, Codex 8.0.

- Every sign-in step uses one mark size and one anchor (the setup form's height), so the mark and card stay put as the steps change. Round 15 had centred each step by its own height, which made them jump.
- One rhythm: the search switcher's row lines up with the dialog's close button, shortcut headings share one gap, empty states put their icon at one height, and the zoom toggle uses one selected style. Scroll shadows are softer, with no hard line.
- Orange steps back: the chat list's unread dot is strong ink and quote bars are neutral. Mention text on its tint uses the darker orange.
- The chat filter has a clear button. New chat has a stable Start chat action with a recipient count. A narrow Calls list keeps the audio call visible and shows video on hover. Chat-list times and Not sent are 13 pt.

What each grader said blocks a 9:

- Opus: History is hard-clipped under every header, slicing a text line in half on most chat screens
- Opus: Surfaces are too flat: card, rail and list separation is 1.06-1.33:1, and dark cards are recessed rather than raised
- Opus: Avatar and team-tile colours repeat (AL/AT, O/HL, Engineering/Contoso), and a team tile matches the accent
- Opus: Selection and focus have no single language: solid accent, grey fill and the 2 px accent composer ring compete
- Opus: Rough edges in secondary screens: settings bottom artefact, crowded meeting system lines, composer scrollbar, toolbar collisions
- Codex: Narrow layouts truncate participant identity and squeeze reading space; the narrow Calls page also removes the visible video action.
- Codex: Thread-card separation is weak, while small utility typography and strongly outlined reactions make the reading hierarchy uneven.
- Codex: Vertical spacing is inconsistent: meeting history leaves large empty gaps, settings rows use different rhythms, and authentication footers jump between states.
- Codex: Extreme-aspect attachment thumbnails are hard to inspect and the long-draft state has no visible expansion affordance.
- Codex: Selected preview controls and explanatory copy do not yet have the same clarity as the strongest dialogs.

## Screenshots

Main screens, round 0 against the final round (round-16):

| Screen | Light | Dark |
| --- | --- | --- |
| chats | [round 0](round-0/light/chats.png) · [round-16](round-16/light/chats.png) | [round 0](round-0/dark/chats.png) · [round-16](round-16/dark/chats.png) |
| channel | [round 0](round-0/light/channel.png) · [round-16](round-16/light/channel.png) | [round 0](round-0/dark/channel.png) · [round-16](round-16/dark/channel.png) |
| calendar | [round 0](round-0/light/calendar.png) · [round-16](round-16/light/calendar.png) | [round 0](round-0/dark/calendar.png) · [round-16](round-16/dark/calendar.png) |
| calls | [round 0](round-0/light/calls.png) · [round-16](round-16/light/calls.png) | [round 0](round-0/dark/calls.png) · [round-16](round-16/dark/calls.png) |
| settings | [round 0](round-0/light/settings.png) · [round-16](round-16/light/settings.png) | [round 0](round-0/dark/settings.png) · [round-16](round-16/dark/settings.png) |
| signin | [round 0](round-0/light/signin.png) · [round-16](round-16/light/signin.png) | [round 0](round-0/dark/signin.png) · [round-16](round-16/dark/signin.png) |
| first-sync-loading | [round 0](round-0/light/first-sync-loading.png) · [round-16](round-16/light/first-sync-loading.png) | [round 0](round-0/dark/first-sync-loading.png) · [round-16](round-16/dark/first-sync-loading.png) |
| chats-narrow | [round 0](round-0/light/chats-narrow.png) · [round-16](round-16/light/chats-narrow.png) | [round 0](round-0/dark/chats-narrow.png) · [round-16](round-16/dark/chats-narrow.png) |

### All committed screens

<details><summary>round-0</summary>

**light:** [account-menu](round-0/light/account-menu.png) · [calendar](round-0/light/calendar.png) · [call-bar](round-0/light/call-bar.png) · [call-docked](round-0/light/call-docked.png) · [calls-narrow](round-0/light/calls-narrow.png) · [calls](round-0/light/calls.png) · [channel-narrow](round-0/light/channel-narrow.png) · [channel](round-0/light/channel.png) · [chat-filter-no-match](round-0/light/chat-filter-no-match.png) · [chat-no-selection](round-0/light/chat-no-selection.png) · [chats-narrow](round-0/light/chats-narrow.png) · [chats-wide](round-0/light/chats-wide.png) · [chats](round-0/light/chats.png) · [composer-images](round-0/light/composer-images.png) · [composer-long-draft](round-0/light/composer-long-draft.png) · [composer-mention](round-0/light/composer-mention.png) · [error-toast](round-0/light/error-toast.png) · [first-sync-empty](round-0/light/first-sync-empty.png) · [first-sync-loading](round-0/light/first-sync-loading.png) · [group-chat](round-0/light/group-chat.png) · [image-preview](round-0/light/image-preview.png) · [join-on-phone](round-0/light/join-on-phone.png) · [new-chat](round-0/light/new-chat.png) · [permission-consent](round-0/light/permission-consent.png) · [search-switcher](round-0/light/search-switcher.png) · [settings](round-0/light/settings.png) · [setup](round-0/light/setup.png) · [shortcuts](round-0/light/shortcuts.png) · [signin-browser](round-0/light/signin-browser.png) · [signin-device-code](round-0/light/signin-device-code.png) · [signin-expired](round-0/light/signin-expired.png) · [signin-failed](round-0/light/signin-failed.png) · [signin-finishing](round-0/light/signin-finishing.png) · [signin](round-0/light/signin.png) · [status-message](round-0/light/status-message.png) · [teams](round-0/light/teams.png)

**dark:** [account-menu](round-0/dark/account-menu.png) · [calendar](round-0/dark/calendar.png) · [call-bar](round-0/dark/call-bar.png) · [call-docked](round-0/dark/call-docked.png) · [calls-narrow](round-0/dark/calls-narrow.png) · [calls](round-0/dark/calls.png) · [channel-narrow](round-0/dark/channel-narrow.png) · [channel](round-0/dark/channel.png) · [chat-filter-no-match](round-0/dark/chat-filter-no-match.png) · [chat-no-selection](round-0/dark/chat-no-selection.png) · [chats-narrow](round-0/dark/chats-narrow.png) · [chats-wide](round-0/dark/chats-wide.png) · [chats](round-0/dark/chats.png) · [composer-images](round-0/dark/composer-images.png) · [composer-long-draft](round-0/dark/composer-long-draft.png) · [composer-mention](round-0/dark/composer-mention.png) · [error-toast](round-0/dark/error-toast.png) · [first-sync-empty](round-0/dark/first-sync-empty.png) · [first-sync-loading](round-0/dark/first-sync-loading.png) · [group-chat](round-0/dark/group-chat.png) · [image-preview](round-0/dark/image-preview.png) · [join-on-phone](round-0/dark/join-on-phone.png) · [new-chat](round-0/dark/new-chat.png) · [permission-consent](round-0/dark/permission-consent.png) · [search-switcher](round-0/dark/search-switcher.png) · [settings](round-0/dark/settings.png) · [setup](round-0/dark/setup.png) · [shortcuts](round-0/dark/shortcuts.png) · [signin-browser](round-0/dark/signin-browser.png) · [signin-device-code](round-0/dark/signin-device-code.png) · [signin-expired](round-0/dark/signin-expired.png) · [signin-failed](round-0/dark/signin-failed.png) · [signin-finishing](round-0/dark/signin-finishing.png) · [signin](round-0/dark/signin.png) · [status-message](round-0/dark/status-message.png) · [teams](round-0/dark/teams.png)

</details>

<details><summary>round-1</summary>

**light:** [account-menu](round-1/light/account-menu.png) · [calendar](round-1/light/calendar.png) · [call-bar](round-1/light/call-bar.png) · [call-docked](round-1/light/call-docked.png) · [calls-narrow](round-1/light/calls-narrow.png) · [calls](round-1/light/calls.png) · [channel-narrow](round-1/light/channel-narrow.png) · [channel](round-1/light/channel.png) · [chat-filter-no-match](round-1/light/chat-filter-no-match.png) · [chat-no-selection](round-1/light/chat-no-selection.png) · [chats-narrow](round-1/light/chats-narrow.png) · [chats-wide](round-1/light/chats-wide.png) · [chats](round-1/light/chats.png) · [composer-images](round-1/light/composer-images.png) · [composer-long-draft](round-1/light/composer-long-draft.png) · [composer-mention](round-1/light/composer-mention.png) · [error-toast](round-1/light/error-toast.png) · [first-sync-empty](round-1/light/first-sync-empty.png) · [first-sync-loading](round-1/light/first-sync-loading.png) · [group-chat](round-1/light/group-chat.png) · [image-preview](round-1/light/image-preview.png) · [join-on-phone](round-1/light/join-on-phone.png) · [new-chat](round-1/light/new-chat.png) · [permission-consent](round-1/light/permission-consent.png) · [search-switcher](round-1/light/search-switcher.png) · [settings](round-1/light/settings.png) · [setup](round-1/light/setup.png) · [shortcuts](round-1/light/shortcuts.png) · [signin-browser](round-1/light/signin-browser.png) · [signin-device-code](round-1/light/signin-device-code.png) · [signin-expired](round-1/light/signin-expired.png) · [signin-failed](round-1/light/signin-failed.png) · [signin-finishing](round-1/light/signin-finishing.png) · [signin](round-1/light/signin.png) · [status-message](round-1/light/status-message.png) · [teams](round-1/light/teams.png)

**dark:** [account-menu](round-1/dark/account-menu.png) · [calendar](round-1/dark/calendar.png) · [call-bar](round-1/dark/call-bar.png) · [call-docked](round-1/dark/call-docked.png) · [calls-narrow](round-1/dark/calls-narrow.png) · [calls](round-1/dark/calls.png) · [channel-narrow](round-1/dark/channel-narrow.png) · [channel](round-1/dark/channel.png) · [chat-filter-no-match](round-1/dark/chat-filter-no-match.png) · [chat-no-selection](round-1/dark/chat-no-selection.png) · [chats-narrow](round-1/dark/chats-narrow.png) · [chats-wide](round-1/dark/chats-wide.png) · [chats](round-1/dark/chats.png) · [composer-images](round-1/dark/composer-images.png) · [composer-long-draft](round-1/dark/composer-long-draft.png) · [composer-mention](round-1/dark/composer-mention.png) · [error-toast](round-1/dark/error-toast.png) · [first-sync-empty](round-1/dark/first-sync-empty.png) · [first-sync-loading](round-1/dark/first-sync-loading.png) · [group-chat](round-1/dark/group-chat.png) · [image-preview](round-1/dark/image-preview.png) · [join-on-phone](round-1/dark/join-on-phone.png) · [new-chat](round-1/dark/new-chat.png) · [permission-consent](round-1/dark/permission-consent.png) · [search-switcher](round-1/dark/search-switcher.png) · [settings](round-1/dark/settings.png) · [setup](round-1/dark/setup.png) · [shortcuts](round-1/dark/shortcuts.png) · [signin-browser](round-1/dark/signin-browser.png) · [signin-device-code](round-1/dark/signin-device-code.png) · [signin-expired](round-1/dark/signin-expired.png) · [signin-failed](round-1/dark/signin-failed.png) · [signin-finishing](round-1/dark/signin-finishing.png) · [signin](round-1/dark/signin.png) · [status-message](round-1/dark/status-message.png) · [teams](round-1/dark/teams.png)

</details>

<details><summary>round-2</summary>

**light:** [account-menu](round-2/light/account-menu.png) · [calendar](round-2/light/calendar.png) · [call-bar](round-2/light/call-bar.png) · [call-docked](round-2/light/call-docked.png) · [calls-narrow](round-2/light/calls-narrow.png) · [calls](round-2/light/calls.png) · [channel-narrow](round-2/light/channel-narrow.png) · [channel](round-2/light/channel.png) · [chat-filter-no-match](round-2/light/chat-filter-no-match.png) · [chat-no-selection](round-2/light/chat-no-selection.png) · [chats-narrow](round-2/light/chats-narrow.png) · [chats-wide](round-2/light/chats-wide.png) · [chats](round-2/light/chats.png) · [composer-images](round-2/light/composer-images.png) · [composer-long-draft](round-2/light/composer-long-draft.png) · [composer-mention](round-2/light/composer-mention.png) · [error-toast](round-2/light/error-toast.png) · [first-sync-empty](round-2/light/first-sync-empty.png) · [first-sync-loading](round-2/light/first-sync-loading.png) · [group-chat](round-2/light/group-chat.png) · [image-preview](round-2/light/image-preview.png) · [join-on-phone](round-2/light/join-on-phone.png) · [new-chat](round-2/light/new-chat.png) · [permission-consent](round-2/light/permission-consent.png) · [search-switcher](round-2/light/search-switcher.png) · [settings](round-2/light/settings.png) · [setup](round-2/light/setup.png) · [shortcuts](round-2/light/shortcuts.png) · [signin-browser](round-2/light/signin-browser.png) · [signin-device-code](round-2/light/signin-device-code.png) · [signin-expired](round-2/light/signin-expired.png) · [signin-failed](round-2/light/signin-failed.png) · [signin-finishing](round-2/light/signin-finishing.png) · [signin](round-2/light/signin.png) · [status-message](round-2/light/status-message.png) · [teams](round-2/light/teams.png)

**dark:** [account-menu](round-2/dark/account-menu.png) · [calendar](round-2/dark/calendar.png) · [call-bar](round-2/dark/call-bar.png) · [call-docked](round-2/dark/call-docked.png) · [calls-narrow](round-2/dark/calls-narrow.png) · [calls](round-2/dark/calls.png) · [channel-narrow](round-2/dark/channel-narrow.png) · [channel](round-2/dark/channel.png) · [chat-filter-no-match](round-2/dark/chat-filter-no-match.png) · [chat-no-selection](round-2/dark/chat-no-selection.png) · [chats-narrow](round-2/dark/chats-narrow.png) · [chats-wide](round-2/dark/chats-wide.png) · [chats](round-2/dark/chats.png) · [composer-images](round-2/dark/composer-images.png) · [composer-long-draft](round-2/dark/composer-long-draft.png) · [composer-mention](round-2/dark/composer-mention.png) · [error-toast](round-2/dark/error-toast.png) · [first-sync-empty](round-2/dark/first-sync-empty.png) · [first-sync-loading](round-2/dark/first-sync-loading.png) · [group-chat](round-2/dark/group-chat.png) · [image-preview](round-2/dark/image-preview.png) · [join-on-phone](round-2/dark/join-on-phone.png) · [new-chat](round-2/dark/new-chat.png) · [permission-consent](round-2/dark/permission-consent.png) · [search-switcher](round-2/dark/search-switcher.png) · [settings](round-2/dark/settings.png) · [setup](round-2/dark/setup.png) · [shortcuts](round-2/dark/shortcuts.png) · [signin-browser](round-2/dark/signin-browser.png) · [signin-device-code](round-2/dark/signin-device-code.png) · [signin-expired](round-2/dark/signin-expired.png) · [signin-failed](round-2/dark/signin-failed.png) · [signin-finishing](round-2/dark/signin-finishing.png) · [signin](round-2/dark/signin.png) · [status-message](round-2/dark/status-message.png) · [teams](round-2/dark/teams.png)

</details>

<details><summary>round-3</summary>

**light:** [account-menu](round-3/light/account-menu.png) · [calendar](round-3/light/calendar.png) · [call-bar](round-3/light/call-bar.png) · [call-docked](round-3/light/call-docked.png) · [calls-narrow](round-3/light/calls-narrow.png) · [calls](round-3/light/calls.png) · [channel-narrow](round-3/light/channel-narrow.png) · [channel](round-3/light/channel.png) · [chat-filter-no-match](round-3/light/chat-filter-no-match.png) · [chat-no-selection](round-3/light/chat-no-selection.png) · [chats-narrow](round-3/light/chats-narrow.png) · [chats-wide](round-3/light/chats-wide.png) · [chats](round-3/light/chats.png) · [composer-images](round-3/light/composer-images.png) · [composer-long-draft](round-3/light/composer-long-draft.png) · [composer-mention](round-3/light/composer-mention.png) · [error-toast](round-3/light/error-toast.png) · [first-sync-empty](round-3/light/first-sync-empty.png) · [first-sync-loading](round-3/light/first-sync-loading.png) · [group-chat](round-3/light/group-chat.png) · [image-preview](round-3/light/image-preview.png) · [join-on-phone](round-3/light/join-on-phone.png) · [new-chat](round-3/light/new-chat.png) · [permission-consent](round-3/light/permission-consent.png) · [search-switcher](round-3/light/search-switcher.png) · [settings](round-3/light/settings.png) · [setup](round-3/light/setup.png) · [shortcuts](round-3/light/shortcuts.png) · [signin-browser](round-3/light/signin-browser.png) · [signin-device-code](round-3/light/signin-device-code.png) · [signin-expired](round-3/light/signin-expired.png) · [signin-failed](round-3/light/signin-failed.png) · [signin-finishing](round-3/light/signin-finishing.png) · [signin](round-3/light/signin.png) · [status-message](round-3/light/status-message.png) · [teams](round-3/light/teams.png)

**dark:** [account-menu](round-3/dark/account-menu.png) · [calendar](round-3/dark/calendar.png) · [call-bar](round-3/dark/call-bar.png) · [call-docked](round-3/dark/call-docked.png) · [calls-narrow](round-3/dark/calls-narrow.png) · [calls](round-3/dark/calls.png) · [channel-narrow](round-3/dark/channel-narrow.png) · [channel](round-3/dark/channel.png) · [chat-filter-no-match](round-3/dark/chat-filter-no-match.png) · [chat-no-selection](round-3/dark/chat-no-selection.png) · [chats-narrow](round-3/dark/chats-narrow.png) · [chats-wide](round-3/dark/chats-wide.png) · [chats](round-3/dark/chats.png) · [composer-images](round-3/dark/composer-images.png) · [composer-long-draft](round-3/dark/composer-long-draft.png) · [composer-mention](round-3/dark/composer-mention.png) · [error-toast](round-3/dark/error-toast.png) · [first-sync-empty](round-3/dark/first-sync-empty.png) · [first-sync-loading](round-3/dark/first-sync-loading.png) · [group-chat](round-3/dark/group-chat.png) · [image-preview](round-3/dark/image-preview.png) · [join-on-phone](round-3/dark/join-on-phone.png) · [new-chat](round-3/dark/new-chat.png) · [permission-consent](round-3/dark/permission-consent.png) · [search-switcher](round-3/dark/search-switcher.png) · [settings](round-3/dark/settings.png) · [setup](round-3/dark/setup.png) · [shortcuts](round-3/dark/shortcuts.png) · [signin-browser](round-3/dark/signin-browser.png) · [signin-device-code](round-3/dark/signin-device-code.png) · [signin-expired](round-3/dark/signin-expired.png) · [signin-failed](round-3/dark/signin-failed.png) · [signin-finishing](round-3/dark/signin-finishing.png) · [signin](round-3/dark/signin.png) · [status-message](round-3/dark/status-message.png) · [teams](round-3/dark/teams.png)

</details>

<details><summary>round-4</summary>

**light:** [account-menu](round-4/light/account-menu.png) · [calendar](round-4/light/calendar.png) · [call-bar](round-4/light/call-bar.png) · [call-docked](round-4/light/call-docked.png) · [calls-narrow](round-4/light/calls-narrow.png) · [calls](round-4/light/calls.png) · [channel-narrow](round-4/light/channel-narrow.png) · [channel](round-4/light/channel.png) · [chat-filter-no-match](round-4/light/chat-filter-no-match.png) · [chat-no-selection](round-4/light/chat-no-selection.png) · [chats-narrow](round-4/light/chats-narrow.png) · [chats-wide](round-4/light/chats-wide.png) · [chats](round-4/light/chats.png) · [composer-images](round-4/light/composer-images.png) · [composer-long-draft](round-4/light/composer-long-draft.png) · [composer-mention](round-4/light/composer-mention.png) · [error-toast](round-4/light/error-toast.png) · [first-sync-empty](round-4/light/first-sync-empty.png) · [first-sync-loading](round-4/light/first-sync-loading.png) · [group-chat](round-4/light/group-chat.png) · [image-preview](round-4/light/image-preview.png) · [join-on-phone](round-4/light/join-on-phone.png) · [new-chat](round-4/light/new-chat.png) · [permission-consent](round-4/light/permission-consent.png) · [search-switcher](round-4/light/search-switcher.png) · [settings](round-4/light/settings.png) · [setup](round-4/light/setup.png) · [shortcuts](round-4/light/shortcuts.png) · [signin-browser](round-4/light/signin-browser.png) · [signin-device-code](round-4/light/signin-device-code.png) · [signin-expired](round-4/light/signin-expired.png) · [signin-failed](round-4/light/signin-failed.png) · [signin-finishing](round-4/light/signin-finishing.png) · [signin](round-4/light/signin.png) · [status-message](round-4/light/status-message.png) · [teams](round-4/light/teams.png)

**dark:** [account-menu](round-4/dark/account-menu.png) · [calendar](round-4/dark/calendar.png) · [call-bar](round-4/dark/call-bar.png) · [call-docked](round-4/dark/call-docked.png) · [calls-narrow](round-4/dark/calls-narrow.png) · [calls](round-4/dark/calls.png) · [channel-narrow](round-4/dark/channel-narrow.png) · [channel](round-4/dark/channel.png) · [chat-filter-no-match](round-4/dark/chat-filter-no-match.png) · [chat-no-selection](round-4/dark/chat-no-selection.png) · [chats-narrow](round-4/dark/chats-narrow.png) · [chats-wide](round-4/dark/chats-wide.png) · [chats](round-4/dark/chats.png) · [composer-images](round-4/dark/composer-images.png) · [composer-long-draft](round-4/dark/composer-long-draft.png) · [composer-mention](round-4/dark/composer-mention.png) · [error-toast](round-4/dark/error-toast.png) · [first-sync-empty](round-4/dark/first-sync-empty.png) · [first-sync-loading](round-4/dark/first-sync-loading.png) · [group-chat](round-4/dark/group-chat.png) · [image-preview](round-4/dark/image-preview.png) · [join-on-phone](round-4/dark/join-on-phone.png) · [new-chat](round-4/dark/new-chat.png) · [permission-consent](round-4/dark/permission-consent.png) · [search-switcher](round-4/dark/search-switcher.png) · [settings](round-4/dark/settings.png) · [setup](round-4/dark/setup.png) · [shortcuts](round-4/dark/shortcuts.png) · [signin-browser](round-4/dark/signin-browser.png) · [signin-device-code](round-4/dark/signin-device-code.png) · [signin-expired](round-4/dark/signin-expired.png) · [signin-failed](round-4/dark/signin-failed.png) · [signin-finishing](round-4/dark/signin-finishing.png) · [signin](round-4/dark/signin.png) · [status-message](round-4/dark/status-message.png) · [teams](round-4/dark/teams.png)

</details>

<details><summary>round-5</summary>

**light:** [account-menu](round-5/light/account-menu.png) · [calendar](round-5/light/calendar.png) · [call-bar](round-5/light/call-bar.png) · [call-docked](round-5/light/call-docked.png) · [calls-narrow](round-5/light/calls-narrow.png) · [calls](round-5/light/calls.png) · [channel-narrow](round-5/light/channel-narrow.png) · [channel](round-5/light/channel.png) · [chat-filter-no-match](round-5/light/chat-filter-no-match.png) · [chat-no-selection](round-5/light/chat-no-selection.png) · [chats-narrow](round-5/light/chats-narrow.png) · [chats-wide](round-5/light/chats-wide.png) · [chats](round-5/light/chats.png) · [composer-images](round-5/light/composer-images.png) · [composer-long-draft](round-5/light/composer-long-draft.png) · [composer-mention](round-5/light/composer-mention.png) · [error-toast](round-5/light/error-toast.png) · [first-sync-empty](round-5/light/first-sync-empty.png) · [first-sync-loading](round-5/light/first-sync-loading.png) · [group-chat](round-5/light/group-chat.png) · [image-preview](round-5/light/image-preview.png) · [join-on-phone](round-5/light/join-on-phone.png) · [new-chat](round-5/light/new-chat.png) · [permission-consent](round-5/light/permission-consent.png) · [search-switcher](round-5/light/search-switcher.png) · [settings](round-5/light/settings.png) · [setup](round-5/light/setup.png) · [shortcuts](round-5/light/shortcuts.png) · [signin-browser](round-5/light/signin-browser.png) · [signin-device-code](round-5/light/signin-device-code.png) · [signin-expired](round-5/light/signin-expired.png) · [signin-failed](round-5/light/signin-failed.png) · [signin-finishing](round-5/light/signin-finishing.png) · [signin](round-5/light/signin.png) · [status-message](round-5/light/status-message.png) · [teams](round-5/light/teams.png)

**dark:** [account-menu](round-5/dark/account-menu.png) · [calendar](round-5/dark/calendar.png) · [call-bar](round-5/dark/call-bar.png) · [call-docked](round-5/dark/call-docked.png) · [calls-narrow](round-5/dark/calls-narrow.png) · [calls](round-5/dark/calls.png) · [channel-narrow](round-5/dark/channel-narrow.png) · [channel](round-5/dark/channel.png) · [chat-filter-no-match](round-5/dark/chat-filter-no-match.png) · [chat-no-selection](round-5/dark/chat-no-selection.png) · [chats-narrow](round-5/dark/chats-narrow.png) · [chats-wide](round-5/dark/chats-wide.png) · [chats](round-5/dark/chats.png) · [composer-images](round-5/dark/composer-images.png) · [composer-long-draft](round-5/dark/composer-long-draft.png) · [composer-mention](round-5/dark/composer-mention.png) · [error-toast](round-5/dark/error-toast.png) · [first-sync-empty](round-5/dark/first-sync-empty.png) · [first-sync-loading](round-5/dark/first-sync-loading.png) · [group-chat](round-5/dark/group-chat.png) · [image-preview](round-5/dark/image-preview.png) · [join-on-phone](round-5/dark/join-on-phone.png) · [new-chat](round-5/dark/new-chat.png) · [permission-consent](round-5/dark/permission-consent.png) · [search-switcher](round-5/dark/search-switcher.png) · [settings](round-5/dark/settings.png) · [setup](round-5/dark/setup.png) · [shortcuts](round-5/dark/shortcuts.png) · [signin-browser](round-5/dark/signin-browser.png) · [signin-device-code](round-5/dark/signin-device-code.png) · [signin-expired](round-5/dark/signin-expired.png) · [signin-failed](round-5/dark/signin-failed.png) · [signin-finishing](round-5/dark/signin-finishing.png) · [signin](round-5/dark/signin.png) · [status-message](round-5/dark/status-message.png) · [teams](round-5/dark/teams.png)

</details>

<details><summary>round-6</summary>

**light:** [account-menu](round-6/light/account-menu.png) · [calendar](round-6/light/calendar.png) · [call-bar](round-6/light/call-bar.png) · [call-docked](round-6/light/call-docked.png) · [calls-narrow](round-6/light/calls-narrow.png) · [calls](round-6/light/calls.png) · [channel-narrow](round-6/light/channel-narrow.png) · [channel](round-6/light/channel.png) · [chat-filter-no-match](round-6/light/chat-filter-no-match.png) · [chat-no-selection](round-6/light/chat-no-selection.png) · [chats-narrow](round-6/light/chats-narrow.png) · [chats-wide](round-6/light/chats-wide.png) · [chats](round-6/light/chats.png) · [composer-images](round-6/light/composer-images.png) · [composer-long-draft](round-6/light/composer-long-draft.png) · [composer-mention](round-6/light/composer-mention.png) · [error-toast](round-6/light/error-toast.png) · [first-sync-empty](round-6/light/first-sync-empty.png) · [first-sync-loading](round-6/light/first-sync-loading.png) · [group-chat](round-6/light/group-chat.png) · [image-preview](round-6/light/image-preview.png) · [join-on-phone](round-6/light/join-on-phone.png) · [new-chat](round-6/light/new-chat.png) · [permission-consent](round-6/light/permission-consent.png) · [search-switcher](round-6/light/search-switcher.png) · [settings](round-6/light/settings.png) · [setup](round-6/light/setup.png) · [shortcuts](round-6/light/shortcuts.png) · [signin-browser](round-6/light/signin-browser.png) · [signin-device-code](round-6/light/signin-device-code.png) · [signin-expired](round-6/light/signin-expired.png) · [signin-failed](round-6/light/signin-failed.png) · [signin-finishing](round-6/light/signin-finishing.png) · [signin](round-6/light/signin.png) · [status-message](round-6/light/status-message.png) · [teams](round-6/light/teams.png)

**dark:** [account-menu](round-6/dark/account-menu.png) · [calendar](round-6/dark/calendar.png) · [call-bar](round-6/dark/call-bar.png) · [call-docked](round-6/dark/call-docked.png) · [calls-narrow](round-6/dark/calls-narrow.png) · [calls](round-6/dark/calls.png) · [channel-narrow](round-6/dark/channel-narrow.png) · [channel](round-6/dark/channel.png) · [chat-filter-no-match](round-6/dark/chat-filter-no-match.png) · [chat-no-selection](round-6/dark/chat-no-selection.png) · [chats-narrow](round-6/dark/chats-narrow.png) · [chats-wide](round-6/dark/chats-wide.png) · [chats](round-6/dark/chats.png) · [composer-images](round-6/dark/composer-images.png) · [composer-long-draft](round-6/dark/composer-long-draft.png) · [composer-mention](round-6/dark/composer-mention.png) · [error-toast](round-6/dark/error-toast.png) · [first-sync-empty](round-6/dark/first-sync-empty.png) · [first-sync-loading](round-6/dark/first-sync-loading.png) · [group-chat](round-6/dark/group-chat.png) · [image-preview](round-6/dark/image-preview.png) · [join-on-phone](round-6/dark/join-on-phone.png) · [new-chat](round-6/dark/new-chat.png) · [permission-consent](round-6/dark/permission-consent.png) · [search-switcher](round-6/dark/search-switcher.png) · [settings](round-6/dark/settings.png) · [setup](round-6/dark/setup.png) · [shortcuts](round-6/dark/shortcuts.png) · [signin-browser](round-6/dark/signin-browser.png) · [signin-device-code](round-6/dark/signin-device-code.png) · [signin-expired](round-6/dark/signin-expired.png) · [signin-failed](round-6/dark/signin-failed.png) · [signin-finishing](round-6/dark/signin-finishing.png) · [signin](round-6/dark/signin.png) · [status-message](round-6/dark/status-message.png) · [teams](round-6/dark/teams.png)

</details>

<details><summary>round-7</summary>

**light:** [account-menu](round-7/light/account-menu.png) · [calendar](round-7/light/calendar.png) · [call-bar](round-7/light/call-bar.png) · [call-docked](round-7/light/call-docked.png) · [calls-narrow](round-7/light/calls-narrow.png) · [calls](round-7/light/calls.png) · [channel-narrow](round-7/light/channel-narrow.png) · [channel](round-7/light/channel.png) · [chat-filter-no-match](round-7/light/chat-filter-no-match.png) · [chat-no-selection](round-7/light/chat-no-selection.png) · [chats-narrow](round-7/light/chats-narrow.png) · [chats-wide](round-7/light/chats-wide.png) · [chats](round-7/light/chats.png) · [composer-images](round-7/light/composer-images.png) · [composer-long-draft](round-7/light/composer-long-draft.png) · [composer-mention](round-7/light/composer-mention.png) · [error-toast](round-7/light/error-toast.png) · [first-sync-empty](round-7/light/first-sync-empty.png) · [first-sync-loading](round-7/light/first-sync-loading.png) · [group-chat](round-7/light/group-chat.png) · [image-preview](round-7/light/image-preview.png) · [join-on-phone](round-7/light/join-on-phone.png) · [new-chat](round-7/light/new-chat.png) · [permission-consent](round-7/light/permission-consent.png) · [search-switcher](round-7/light/search-switcher.png) · [settings](round-7/light/settings.png) · [setup](round-7/light/setup.png) · [shortcuts](round-7/light/shortcuts.png) · [signin-browser](round-7/light/signin-browser.png) · [signin-device-code](round-7/light/signin-device-code.png) · [signin-expired](round-7/light/signin-expired.png) · [signin-failed](round-7/light/signin-failed.png) · [signin-finishing](round-7/light/signin-finishing.png) · [signin](round-7/light/signin.png) · [status-message](round-7/light/status-message.png) · [teams](round-7/light/teams.png)

**dark:** [account-menu](round-7/dark/account-menu.png) · [calendar](round-7/dark/calendar.png) · [call-bar](round-7/dark/call-bar.png) · [call-docked](round-7/dark/call-docked.png) · [calls-narrow](round-7/dark/calls-narrow.png) · [calls](round-7/dark/calls.png) · [channel-narrow](round-7/dark/channel-narrow.png) · [channel](round-7/dark/channel.png) · [chat-filter-no-match](round-7/dark/chat-filter-no-match.png) · [chat-no-selection](round-7/dark/chat-no-selection.png) · [chats-narrow](round-7/dark/chats-narrow.png) · [chats-wide](round-7/dark/chats-wide.png) · [chats](round-7/dark/chats.png) · [composer-images](round-7/dark/composer-images.png) · [composer-long-draft](round-7/dark/composer-long-draft.png) · [composer-mention](round-7/dark/composer-mention.png) · [error-toast](round-7/dark/error-toast.png) · [first-sync-empty](round-7/dark/first-sync-empty.png) · [first-sync-loading](round-7/dark/first-sync-loading.png) · [group-chat](round-7/dark/group-chat.png) · [image-preview](round-7/dark/image-preview.png) · [join-on-phone](round-7/dark/join-on-phone.png) · [new-chat](round-7/dark/new-chat.png) · [permission-consent](round-7/dark/permission-consent.png) · [search-switcher](round-7/dark/search-switcher.png) · [settings](round-7/dark/settings.png) · [setup](round-7/dark/setup.png) · [shortcuts](round-7/dark/shortcuts.png) · [signin-browser](round-7/dark/signin-browser.png) · [signin-device-code](round-7/dark/signin-device-code.png) · [signin-expired](round-7/dark/signin-expired.png) · [signin-failed](round-7/dark/signin-failed.png) · [signin-finishing](round-7/dark/signin-finishing.png) · [signin](round-7/dark/signin.png) · [status-message](round-7/dark/status-message.png) · [teams](round-7/dark/teams.png)

</details>

<details><summary>round-8</summary>

**light:** [account-menu](round-8/light/account-menu.png) · [calendar](round-8/light/calendar.png) · [call-bar](round-8/light/call-bar.png) · [call-docked](round-8/light/call-docked.png) · [calls-narrow](round-8/light/calls-narrow.png) · [calls](round-8/light/calls.png) · [channel-narrow](round-8/light/channel-narrow.png) · [channel](round-8/light/channel.png) · [chat-filter-no-match](round-8/light/chat-filter-no-match.png) · [chat-no-selection](round-8/light/chat-no-selection.png) · [chats-narrow](round-8/light/chats-narrow.png) · [chats-wide](round-8/light/chats-wide.png) · [chats](round-8/light/chats.png) · [composer-images](round-8/light/composer-images.png) · [composer-long-draft](round-8/light/composer-long-draft.png) · [composer-mention](round-8/light/composer-mention.png) · [error-toast](round-8/light/error-toast.png) · [first-sync-empty](round-8/light/first-sync-empty.png) · [first-sync-loading](round-8/light/first-sync-loading.png) · [group-chat](round-8/light/group-chat.png) · [image-preview](round-8/light/image-preview.png) · [join-on-phone](round-8/light/join-on-phone.png) · [new-chat](round-8/light/new-chat.png) · [permission-consent](round-8/light/permission-consent.png) · [search-switcher](round-8/light/search-switcher.png) · [settings](round-8/light/settings.png) · [setup](round-8/light/setup.png) · [shortcuts](round-8/light/shortcuts.png) · [signin-browser](round-8/light/signin-browser.png) · [signin-device-code](round-8/light/signin-device-code.png) · [signin-expired](round-8/light/signin-expired.png) · [signin-failed](round-8/light/signin-failed.png) · [signin-finishing](round-8/light/signin-finishing.png) · [signin](round-8/light/signin.png) · [status-message](round-8/light/status-message.png) · [teams](round-8/light/teams.png)

**dark:** [account-menu](round-8/dark/account-menu.png) · [calendar](round-8/dark/calendar.png) · [call-bar](round-8/dark/call-bar.png) · [call-docked](round-8/dark/call-docked.png) · [calls-narrow](round-8/dark/calls-narrow.png) · [calls](round-8/dark/calls.png) · [channel-narrow](round-8/dark/channel-narrow.png) · [channel](round-8/dark/channel.png) · [chat-filter-no-match](round-8/dark/chat-filter-no-match.png) · [chat-no-selection](round-8/dark/chat-no-selection.png) · [chats-narrow](round-8/dark/chats-narrow.png) · [chats-wide](round-8/dark/chats-wide.png) · [chats](round-8/dark/chats.png) · [composer-images](round-8/dark/composer-images.png) · [composer-long-draft](round-8/dark/composer-long-draft.png) · [composer-mention](round-8/dark/composer-mention.png) · [error-toast](round-8/dark/error-toast.png) · [first-sync-empty](round-8/dark/first-sync-empty.png) · [first-sync-loading](round-8/dark/first-sync-loading.png) · [group-chat](round-8/dark/group-chat.png) · [image-preview](round-8/dark/image-preview.png) · [join-on-phone](round-8/dark/join-on-phone.png) · [new-chat](round-8/dark/new-chat.png) · [permission-consent](round-8/dark/permission-consent.png) · [search-switcher](round-8/dark/search-switcher.png) · [settings](round-8/dark/settings.png) · [setup](round-8/dark/setup.png) · [shortcuts](round-8/dark/shortcuts.png) · [signin-browser](round-8/dark/signin-browser.png) · [signin-device-code](round-8/dark/signin-device-code.png) · [signin-expired](round-8/dark/signin-expired.png) · [signin-failed](round-8/dark/signin-failed.png) · [signin-finishing](round-8/dark/signin-finishing.png) · [signin](round-8/dark/signin.png) · [status-message](round-8/dark/status-message.png) · [teams](round-8/dark/teams.png)

</details>

<details><summary>round-16</summary>

**light:** [account-menu](round-16/light/account-menu.png) · [calendar](round-16/light/calendar.png) · [call-bar](round-16/light/call-bar.png) · [call-docked](round-16/light/call-docked.png) · [calls-narrow](round-16/light/calls-narrow.png) · [calls](round-16/light/calls.png) · [channel-narrow](round-16/light/channel-narrow.png) · [channel](round-16/light/channel.png) · [chat-filter-no-match](round-16/light/chat-filter-no-match.png) · [chat-no-selection](round-16/light/chat-no-selection.png) · [chats-narrow](round-16/light/chats-narrow.png) · [chats-wide](round-16/light/chats-wide.png) · [chats](round-16/light/chats.png) · [composer-images](round-16/light/composer-images.png) · [composer-long-draft](round-16/light/composer-long-draft.png) · [composer-mention](round-16/light/composer-mention.png) · [error-toast](round-16/light/error-toast.png) · [first-sync-empty](round-16/light/first-sync-empty.png) · [first-sync-loading](round-16/light/first-sync-loading.png) · [group-chat](round-16/light/group-chat.png) · [image-preview](round-16/light/image-preview.png) · [join-on-phone](round-16/light/join-on-phone.png) · [new-chat](round-16/light/new-chat.png) · [permission-consent](round-16/light/permission-consent.png) · [search-switcher](round-16/light/search-switcher.png) · [settings](round-16/light/settings.png) · [setup](round-16/light/setup.png) · [shortcuts](round-16/light/shortcuts.png) · [signin-browser](round-16/light/signin-browser.png) · [signin-device-code](round-16/light/signin-device-code.png) · [signin-expired](round-16/light/signin-expired.png) · [signin-failed](round-16/light/signin-failed.png) · [signin-finishing](round-16/light/signin-finishing.png) · [signin](round-16/light/signin.png) · [status-message](round-16/light/status-message.png) · [teams](round-16/light/teams.png)

**dark:** [account-menu](round-16/dark/account-menu.png) · [calendar](round-16/dark/calendar.png) · [call-bar](round-16/dark/call-bar.png) · [call-docked](round-16/dark/call-docked.png) · [calls-narrow](round-16/dark/calls-narrow.png) · [calls](round-16/dark/calls.png) · [channel-narrow](round-16/dark/channel-narrow.png) · [channel](round-16/dark/channel.png) · [chat-filter-no-match](round-16/dark/chat-filter-no-match.png) · [chat-no-selection](round-16/dark/chat-no-selection.png) · [chats-narrow](round-16/dark/chats-narrow.png) · [chats-wide](round-16/dark/chats-wide.png) · [chats](round-16/dark/chats.png) · [composer-images](round-16/dark/composer-images.png) · [composer-long-draft](round-16/dark/composer-long-draft.png) · [composer-mention](round-16/dark/composer-mention.png) · [error-toast](round-16/dark/error-toast.png) · [first-sync-empty](round-16/dark/first-sync-empty.png) · [first-sync-loading](round-16/dark/first-sync-loading.png) · [group-chat](round-16/dark/group-chat.png) · [image-preview](round-16/dark/image-preview.png) · [join-on-phone](round-16/dark/join-on-phone.png) · [new-chat](round-16/dark/new-chat.png) · [permission-consent](round-16/dark/permission-consent.png) · [search-switcher](round-16/dark/search-switcher.png) · [settings](round-16/dark/settings.png) · [setup](round-16/dark/setup.png) · [shortcuts](round-16/dark/shortcuts.png) · [signin-browser](round-16/dark/signin-browser.png) · [signin-device-code](round-16/dark/signin-device-code.png) · [signin-expired](round-16/dark/signin-expired.png) · [signin-failed](round-16/dark/signin-failed.png) · [signin-finishing](round-16/dark/signin-finishing.png) · [signin](round-16/dark/signin.png) · [status-message](round-16/dark/status-message.png) · [teams](round-16/dark/teams.png)

</details>

## Outcome

- **Run 1 (rounds 0-8):** both graders went from 6.0 (Opus) and 6.5 (Codex) at the baseline to 8.0 each.
- **Run 2 (rounds 9-16):** the goal was 9.0 from both graders. It wasn't reached. Both graders held at 8.0 from round 11 to round 16 (Codex dipped to 7.5 in round 10), so the run stopped at round 16, the limit.

Why it plateaued:

1. **Blockers that need features.** Every round, both graders named blockers that require new features, which were out of scope:
   - a collapsible conversation list for narrow windows (names truncate at 760 pt);
   - a richer Calendar detail page (attendees, response, agenda from Graph);
   - a way to reach other chats while docked beside a call.
2. **Contradictory advice between rounds.** Several items flipped back and forth:
   - Scroll-edge fades (one grader asked for them; the other measured them as contrast failures).
   - Thumbnail crop versus fit.
   - Hover-only call buttons versus always-visible ones.
   - Header actions at the pane edge versus over the content column.

   Each version scored the same, give or take half a point.
3. **The capture font.** Inter's wide hyphen (a property of the font file, also seen in Firefox) costs typography points in most rounds.

Run 2 still moved real measurements, even where the overall stayed at 8:

- Text, control edges and danger red now reach WCAG 2.2 AA on every surface, and a unit test enforces it.
- egui's own scroll-edge fade, which left faded text below 4.5:1, is off.
- Dialogs share one frame, and running text shares one line height.
- Several latent bugs surfaced and were fixed:
  - the sidebar divider was painted over and invisible;
  - links drew in body text colour;
  - the composer's width cap had no effect;
  - the search field jumped while typing.

| Check | Baseline (main, a4ebc91) | End of run 1 (b8f33cc) | End of run 2 (c20aaee) |
| --- | --- | --- | --- |
| `cargo build --locked --release` | pass | pass | pass |
| `cargo test --locked --all-targets` | pass, 181 tests | pass, 186 tests | pass, 186 tests |
| `cargo clippy --locked --all-targets -- -D warnings` | pass | pass | pass |
| `cargo fmt --all --check` | pass | pass | pass |
| `packaging/generate-notices.sh --check` | pass | pass | pass |

## Capture notes

- **Font.** Screenshots render with Wren's bundled Inter, so they're reproducible. Normal runs use the platform's interface font. Inter's hyphen glyph is wide by design (it renders the same in Firefox), which is why "On-call" looks spaced in the shots. Both graders flagged it in most rounds; it isn't a layout bug, and the app was not changed for it.
- **Focus.** Under Xvfb with no window manager the window never receives focus, and egui only draws focus rings and the caret in a focused window. Up to round 6 the shots showed text fields as unfocused. From round 7, screenshot mode treats the window as focused (`raw_input_hook` in `main.rs`).
- **Timing.** Software GL takes a few seconds to the first frame, and dialogs fade in after that. The first round-0 capture caught dialogs mid fade-in, so it was redone with a 9 s delay before grading.
- **Pointer.** The pointer rests at the centre of the virtual screen, so some shots show hover states, such as the message toolbar or the code sign-in tooltip. Once round 5 centred the sign-in page vertically, that tooltip appears in the sign-in shots.
- **New `--page` states** for the review: sign-in steps and failures, Teams with no channel, no chat open, no filter matches, first-sync loading and empty, a failed send, and the consent dialog.

- **Run 2 capture.** Rounds 9-15 were captured to `/tmp/wren-hill-climb/round-N/`, outside the repository. Only round 16, the last round, is committed.
- **Fades.** Until round 15, egui's default `ScrollStyle::fade` (strength 0.5 over 20 pt) faded text at every scroll edge. It is now off.

## Still blocking a 9 (round 16)

### Features the graders called blocking (not built, by instruction)

- **Narrow windows:** a collapsible conversation list or list/detail view below about 900 pt. Names truncate in chats, channels and calls at 760 pt. Both graders, every round.
- **Calendar detail:** attendees, your response, and the agenda. These need more Graph data than Wren reads today. Opus, most rounds.
- **Docked calls:** a way to reach other chats while docked beside a call. Opus, rounds 9-11.

### Interface items left in the final grades

- Opus:
  - History is hard-clipped under the header, cutting a line in half.
  - Surfaces are still too flat: cards, rail and list separate at 1.06-1.33:1, and dark cards sit below the page.
  - Avatar and team-tile hues repeat, and one tile is close to the accent.
  - Selection and focus treatments compete.
  - The Settings bottom edge, crowded meeting system lines and the composer scrollbar are rough.
- Codex:
  - Thread-card separation is weak, and reaction outlines are strong.
  - Small utility text.
  - Spacing in meeting history and Settings is uneven.
  - Extreme-aspect thumbnails are hard to inspect.
  - The long draft has no expand affordance.

## Decisions

- **Thumbnails:** pasted images fit their tile whole, with a 12 pt minimum short side (round 10). The UI tests check proportions and the minimum. The graders were split between this and cropping.
- **Hover toolbar:** it keeps all six reactions. Three are on the bar and three are in More, together with Copy, Edit and Delete.
- **Call bar:** it has no elapsed timer. Wren follows presence, not call media, so a timer would only be an estimate.

## Grades

Each grader's full report is in [`grades/`](grades/), with every issue and the measured contrast ratios.
